
# Lunar: Task Runner for Lune

A task runner that runs [Lune](https://github.com/lune-org/lune) scripts.

Run any lune script in your workspace, or link to repos, directories, or other local files to run shared scripts. It's as easy as `lunar script args`!

Use `lunar help` to show a list of all tasks it can run in the current directory.

Use Lunar as an alternative to other task runners or makefile tools.

## Installation

Add `lunar` to your `aftman.toml` (or equivalent toolchain manager):
```toml
lunar = "corecii/lunar@0.1.0"
```

OR download it from [the Releases page](https://github.com/corecii/lunar/releases/latest) and put it in your `PATH`.

## Cheat Sheet

I figure the easiest way to get across what Lunar can do is to give some examples.

### A Simple Script

Lunar will show any script under `lune`, `.lune`, `lunar`, or `.lunar` directories as a runnable task without any extra configuration.

### A Script with a Description and Argument Help

Add a comment anywhere in the script like so:

```lua
--[=[ lunar
	about = "your description here"
	args = "your arguments here"
]=]
```

Both fields are optional.

### A Script with Multiple Tasks

Define subtasks in the script like so:

```lua
--[=[ lunar
	[tasks.task_name]
		args = "your arguments here"
		about = "your description here"

	[tasks.another_task]
		# Fields are optional

	[tasks.yet_another_task]
]=]
```

You can also hide the main task:

```lua
	hide = true

	[tasks.task_name]
		args = "your arguments here"
		about = "your description here"
```

### Link to a Script

Create a `your_script_name.lunar.toml` file in any of `lune`, `.lune`, `lunar`, or `.lunar` directories, like so:

```toml
	script = "path/to/your/script.luau"
```

You can also give it a name to override the `.lunar.toml` name:

```toml
	name = "my_task"
	script = "path/to/your/script.luau"
```

### Link to a Directory

When you link to a directory, Lunar will search `lune`, `.lune`, `lunar`, or `.lunar` for scripts in that directory, just like it does in the working directory.

Create a `any_hame_here.lunar.toml` file, like so:

```toml
	[directory]
		path = "path/to/your/directory"
```

You can also filter to only show specific tasks:

```toml
	[directory]
		path = "path/to/your/directory"
		tasks = ["task_name", "another_task"]
```

Or give all tasks in this directory a prefix:

```toml
	[directory]
		path = "path/to/your/directory"
		prefix = "your-prefix-"
```

### Link to a Repo

Linking to a repo is a lot like linking to a directory:

```toml
	[repo]
		url = "https://github.com/owner/repo"
```

You can also specify a branch or tag with the `tag` field:

```toml
	[repo]
		url = "https://github.com/owner/repo"
		tag = "main"
```

Or, you can specify a hash with the `hash` field:

```toml
	[repo]
		url = "https://github.com/owner/repo"
		hash = "1ddd252d7161f57bccfd171788801c3e41302a1c"
		# Must be a full 40-character hash
```

Repos support all of the script filtering and prefixing fields that directories do.

### Using a Single Script from a Repo

```toml
	# my_repo_task.lunar.toml
	[repo]
		url = "https://github.com/owner/repo"

		script = "script_name"
```

This will take on the name of the `lunar.toml` file, so the task will be named `my_repo_task`.
If you want to give it a different name, use the top-level `name` field:

```toml
	# my_repo_task.lunar.toml
	name = "my_task"

	[repo]
		url = "https://github.com/owner/repo"
		
		script = "script_name"
```

With the name field, this will be renamed to `my_task`.

## Repo Setup Scripts

If you're publishing a repo with Lune scripts for Lunar, and they depend on prerequisites, you can include a
`lunar-setup.luau` script in the top-level directory.

Whenever a repo is downloaded, `lunar-setup.luau` will be run to set it up for the first time. You can invoke package managers, grab some stuff from git, etc. If your script fails, the repo will be considered a failure and will try again next time.

`lunar-setup.luau` is run with the repo as the current working directory.

## Warning on Repo Tasks and Security

Lune is powerful. It can run any command on your system and modify the filesystem. Using any Lune script should be treated as seriously as running any executable.

Lunar will run any task you give it. It will also run setup scripts automatically after downloading tasks from the Internet.

Lunar has a simple trust procedure to prevent mistakes, but we are not liable for any damage caused by downloading and running malicious scripts that you instruct it to download. You should ensure every script comes from someone you trust.

## Lune, Git and Misc. Information

### Caching

Lunar caches each repo+hash the first time it's used. If the current hash for a tag or branch can't be fetched from the internet, the cached hash and data for that hash will be used.

Lunar caches each repo by doing a shallow clone of the specific git hash that was targeted. If targeting a tag or a branch, the get hash is fetched first. When specifying a tag or branch, Lunar will automatically download newer versions of the repo when the tag or branch points to a newer commit. **Cached repos are not automatically cleaned up,** but they are shallow clones so they _don't_ contain the full history.

Lunar by default keeps data in `data_local/.lunar-tr` . You can change this directory with the `LUNAR_TR_DIR` environment variable. See [dirs documentation](https://docs.rs/dirs/5.0.1/dirs/fn.data_local_dir.html) for information about where `data_local` is on your system.

### Versions

Lunar doesn't bundle Lune or Git. It uses the versions installed on your system. If we bundled these, we'd have to make a new release every time Lune or Git updated.

Because we use the system Git executable, Git authentication is handled by Git and its global settings. If you want Lunar to be able to download private repositories, just log into your account as part of your global Git settings.

We recommend keeping both Lune and Git up-to-date. Git, in particular, needs to be at least version `2.24` or so for `lunar` to work with remote repos.

Lune runs approximately the following git command to cache repos: (see [this](https://stackoverflow.com/a/43136160) StackOverflow answer for more details)
```sh
git init
git remote add origin <url>
git fetch --depth 1 origin <sha1>
git checkout FETCH_HEAD
```

### Why the name?

`Lune Run` -> `Luner[un]` -> `Luner` -> `Lunar`.

"Luner" sounds silly and "Lunerun" is too verbose!

I'm open to other name suggestions.