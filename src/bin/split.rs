use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use ::git2 as git;

use ::repo_spliter::{
    cli::{cli, Cli, Remove},
    Error, Result,
};

fn main() {
    let cli = cli();
    split(cli).unwrap();
}

const BRANCH_PREFIX: &str = "temp_split_";
const REMOTE_PREFIX: &str = "temp_split_";

fn split(cli: &Cli) -> Result<()> {
    println!("{:#?}", cli);

    let repo_pb = &PathBuf::from(&cli.repo);
    let path_pb = &repo_pb.join(&cli.path);
    let target_pb = &cli.local.as_ref().map(PathBuf::from);

    assert!(repo_pb.is_dir() && repo_pb.exists());
    assert!(path_pb.is_dir() && path_pb.exists());

    if let Some(target_pb) = target_pb {
        if !target_pb.exists() {
            fs::create_dir_all(target_pb)?;
        } else {
            assert!(target_pb.is_dir());
        }
    }

    let repo_path = &fs::canonicalize(repo_pb)?;
    let path_path = &fs::canonicalize(path_pb)?;
    let target_path = &target_pb.as_ref().map(fs::canonicalize).transpose()?;

    // fit for windows
    let path_rel = &cli.path.replace('\\', "/");

    let repo_git = &fuck_windows(repo_path)?;
    let path_git = &fuck_windows(path_path)?;
    // let target_git = &fuck_windows(target_path)?;

    let repo = git::Repository::open(repo_path)?;

    let mut options = git::StatusOptions::new();
    options.include_untracked(true);

    for status in repo.statuses(Some(&mut options))?.into_iter() {
        let code = status.status();
        if !code.is_ignored() && code != git::Status::CURRENT {
            return Err(git::Error::new(
                git::ErrorCode::Uncommitted,
                git::ErrorClass::Filter,
                format!(
                    "you have unstaged changes: {:?} {:?}",
                    status.path(),
                    status.status()
                ),
            )
            .into());
        }
    }

    if let Some(target_path) = target_path {
        match git::Repository::open(target_path) {
            Ok(_) => (),
            Err(e) if e.code() == git::ErrorCode::NotFound => {
                git::Repository::init(target_path)?;
            }
            Err(e) => {
                return Err(e.into());
            }
        };
    }

    let temp_branch_name = &{
        let mut temp_branch_name = BRANCH_PREFIX.to_string();
        while repo
            .find_branch(&temp_branch_name, git::BranchType::Local)
            .is_ok()
        {
            temp_branch_name.push('_');
        }
        temp_branch_name
    };
    let temp_remote_name = &{
        let mut temp_remote_name = REMOTE_PREFIX.to_string();
        while repo.find_remote(&temp_remote_name).is_ok() {
            temp_remote_name.push('_');
        }
        temp_remote_name
    };

    println!("waiting for subtree to finish ...");

    execute(
        Command::new("git")
            .current_dir(repo_path)
            .arg("subtree")
            .arg("split")
            .arg("-P")
            .arg(path_rel)
            .arg("-b")
            .arg(temp_branch_name),
    )?;

    let mut branch = repo.find_branch(temp_branch_name, git::BranchType::Local)?;

    println!("subtree finished.");

    if let Some(target_path) = target_path {
        println!(
            "local adding '{}' to '{:?}' ...",
            temp_branch_name, target_path
        );

        execute(
            Command::new("git")
                .current_dir(target_path)
                .arg("pull")
                .arg(repo_git)
                .arg(temp_branch_name),
        )?;

        println!("local added.");
    }

    if let Some(remote) = &cli.remote {
        println!("remote adding '{}' to '{}' ...", temp_branch_name, remote);

        if let Some(target_path) = target_path {
            execute(
                Command::new("git")
                    .current_dir(target_path)
                    .arg("remote")
                    .arg("add")
                    .arg("origin")
                    .arg(remote),
            )?;

            execute(
                Command::new("git")
                    .current_dir(target_path)
                    .arg("branch")
                    .arg("-M")
                    .arg("main"),
            )?;

            execute(
                Command::new("git")
                    .current_dir(target_path)
                    .arg("push")
                    .arg("-u")
                    .arg("origin")
                    .arg("main"),
            )?;
        } else {
            execute(
                Command::new("git")
                    .current_dir(repo_path)
                    .arg("remote")
                    .arg("add")
                    .arg(temp_remote_name)
                    .arg(remote),
            )?;

            execute(
                Command::new("git")
                    .current_dir(repo_path)
                    .arg("push")
                    .arg(temp_remote_name)
                    .arg(format!("{temp_branch_name}:main")),
            )?;

            execute(
                Command::new("git")
                    .current_dir(repo_path)
                    .arg("remote")
                    .arg("remove")
                    .arg(temp_remote_name),
            )?;
        }

        println!("remote added.");
    }

    branch.delete()?;
    println!("branch {temp_branch_name} deleted.");

    drop(branch);
    drop(repo);

    match cli.remove {
        Remove::Nothing => {
            println!("remove nothing.");
            return Ok(());
        }
        Remove::Commit => {
            println!("rm -rf {:?} ...", path_pb);
            std::fs::remove_dir_all(path_pb)?;

            execute(
                Command::new("git")
                    .current_dir(repo_path)
                    .arg("add")
                    .arg("."),
            )?;

            execute(
                Command::new("git")
                    .current_dir(repo_path)
                    .arg("commit")
                    .arg("-m")
                    .arg(format!("remove {}", path_git)),
            )?;

            println!("done.");
        }
        Remove::Prune => {
            println!("pruning {path_rel} from {repo_path:?} ...");

            execute(
                Command::new("git")
                    .current_dir(repo_path)
                    .arg("filter-branch")
                    .arg("--index-filter")
                    .arg(format!("git rm -rf --cached --ignore-unmatch {path_rel}"))
                    .arg("--prune-empty")
                    .arg("--")
                    .arg("--all"),
            )?;

            let for_each_ref = Command::new("git")
                .current_dir(repo_path)
                .arg("for-each-ref")
                .arg("--format=%(refname)")
                .arg("refs/original/")
                .output()?;
            for refname in String::from_utf8_lossy(&for_each_ref.stdout).lines() {
                execute(
                    Command::new("git")
                        .current_dir(repo_path)
                        .arg("update-ref")
                        .arg("-d")
                        .arg(refname),
                )?;
            }

            execute(
                Command::new("git")
                    .current_dir(repo_path)
                    .arg("reflog")
                    .arg("expire")
                    .arg("--expire=now")
                    .arg("--all"),
            )?;

            execute(
                Command::new("git")
                    .current_dir(repo_path)
                    .arg("gc")
                    .arg("--aggressive")
                    .arg("--prune=now"),
            )?;
        }
    };

    if !cli.keep {
        return Ok(());
    }

    if let Some(remote) = &cli.remote {
        println!("keeping remote ...");

        execute(
            Command::new("git")
                .current_dir(repo_path)
                .arg("submodule")
                .arg("add")
                .arg(remote)
                .arg(path_rel),
        )?;

        execute(
            Command::new("git")
                .current_dir(repo_path)
                .arg("add")
                .arg("."),
        )?;

        execute(
            Command::new("git")
                .current_dir(repo_path)
                .arg("commit")
                .arg("-m")
                .arg(format!("refactor: add {}", path_rel)),
        )?;

        println!("done.");
    } else {
        println!("cannot keep remote without remote.");
    }

    Ok(())
}

fn execute(cmd: &mut Command) -> Result<()> {
    let exitstatus = cmd.spawn()?.wait()?;
    if !exitstatus.success() {
        Err(Error::Execute(format!("{cmd:?}"), exitstatus))
    } else {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn fuck_windows(path: &Path) -> Result<String> {
    Ok(format!(
        "file:///{}",
        path.to_str()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid char in path",
            ))?
            .to_string()
            .trim_start_matches(r"\\?\")
            .replace('\\', "/")
    ))
}

#[cfg(not(target_os = "windows"))]
fn fuck_windows(path: &Path) -> Result<String> {
    Ok(path
        .to_str()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid char in path",
        ))?
        .to_string())
}
