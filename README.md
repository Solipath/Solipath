# Solipath
Solipath is a WIP tool for managing programming languages and tooling required to develop software at a project level.

Think NPM or Gradle, but for downloading a specific NPM or Gradle version, and setting the proper environment variables temporarily in order to run commands for those tools.

## Usage
When using a project setup with solipath, all you need to do is prefix the command you want to run with the solipath script.
### Bash
```bash
./solipath gradle build
```
### Command Prompt
```
solipath gradle build
```
As of right now, solipath simply appends to the path, if the computer already has a dependency in the path, I'm not certain which one would take precedence.

## How it works
### `solipath.sh` and `solipath.bat` files 
These are short scripts that will download a larger shell/batch script that will download a version of the solipath executable for your platform (early on will likely only support x86_64 Windows, Linux, and macOS mostly because I don't really have access to Arm variants of any of those). Will post a link here once the shell script is completed.

### `solipath.json` file
The `solipath.json` file contains the dependencies that you want to download and set on your path. Early on there will only be a couple dependencies and versions you will be able to use with this, but this list will grow over time.

```json
[
	{"name": "java", "version": "11.0.10+9"},
	{"name": "gradle", "version":"6.7"}
]
```
### `install_instructions.json` file
The `install_instructions.json` files contain links to download and environment variables to set so these dependencies are on the path. These are automatically downloaded by solipath. 
```json
{
	"downloads": [
		{
			"url": "https://golang.org/dl/go1.16.linux-amd64.tar.gz",
			"destination_directory": "1.16",
			"platform_filters": [{"os": "linux", "arch": "x86_64"}]
		},
		{
			"url": "https://golang.org/dl/go1.16.windows-amd64.zip",
			"destination_directory": "1.16",
			"platform_filters": [{"os": "windows", "arch": "x86_64"}]
    	},
		{
			"url": "https://golang.org/dl/go1.16.darwin-amd64.tar.gz",
			"destination_directory": "1.16",
			"platform_filters": [{"os": "macos", "arch": "x86_64"}]
		}
	],
	"environment_variables": [
		{"name": "PATH", "relative_path": "1.16/go/bin"}
	]
}
```
### Running solipath
When you run solipath as described in the usage above, solipath will read the `solipath.json` file, and download `install_instructions.json` files for each dependency name and version. Once this is finished, solipath will execute any commands that are forwarded to it. After solipath is finished running, the environment variables that were set will not persist.

### Downloads
All files that are downloaded will be placed in ~/solipath

## Future Capabilities
### Install instruction templates
Generally download links locations and environment variables rarely ever change much between versions. The plan is to introduce templates, where `install_instructions.json` just needs to contain a reference to a template file and some variables for find/replace. This should reduce most `install_instructions.json` to just a line or two of json.

### Install Commands
Some programming languages/tooling can't be installed by decompressing a file and setting an environment variable for all operatings systems I want to support. For these there might be some commands that need to be run in order to finish an install. This will be another feature that will eventually get to `install_instructions.json`.