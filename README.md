# repo-spliter

A specialized tool for extracting directories from Git repositories while preserving history. Offers numerous configurable options and cross-platform support (Windows, Unix; macOS untested).

```
Usage: split.exe [OPTIONS] <REPO> <PATH> [REMOVE]

Arguments:
  <REPO>     Path to source Git repository
  <PATH>     Relative path to target subdirectory
  [REMOVE]   Post-split cleanup action [default: nothing]
             Possible values:
             - nothing: Preserve original directory
             - commit:  Remove directory in new commit
             - prune:  Purge directory from history

Options:
  -l, --local <PATH>    Output path for new repository
  -r, --remote <URL>    Remote repository URL to set
  -k, --keep            Convert original directory to submodule (requires removal)
  -h, --help            Display help information
```
