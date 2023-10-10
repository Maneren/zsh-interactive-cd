# zsh-interactive-cd

## Demo

![demo](demo.gif)

## Usage

Press TAB for completion as usual, it'll launch skim (fzf like interface). Check skim's readme for more search syntax usage.

## Configuration

Set enviroment variable with given name to `1` or `true`

| option                 | effect                                |
| ---------------------- | ------------------------------------- |
| `zic-case-insensitive` | ignores case                          |
| `zic-ignore-dots`      | includes hidden files in all searches |

## Installation

### 1. Install plugin

Download the binary from Gitub releases or compile it from source with `cargo` (install that via [`rustup`](https://rustup.rs/))

<details>
  <summary style="font-size:1.25rem;">zinit</summary>

Add one of following snippets to your `.zshrc`:

Download binary

```zsh
zinit ice make'download'
zinit load Maneren/zsh-interactive-cd
```

or compile

```zsh
zinit ice make'build'
zinit load Maneren/zsh-interactive-cd
```

</details>

<details>
  <summary style="font-size:1.25rem;">oh-my-zsh</summary>

1. Download the plugin

```sh
git clone https://github.com/Maneren/zsh-interactive-cd $ZSH_CUSTOM/plugins/zsh-interactive-cd
```

2. Download binary

```sh
make download -C $ZSH_CUSTOM/plugins/zsh-interactive-cd
```

or compile

```sh
make build -C $ZSH_CUSTOM/plugins/zsh-interactive-cd
```

3. And add `zsh-interactive-cd` to plugins list in `.zshrc`

```zsh
plugins=(
  ...

  zsh-interactive-cd
)
```

</details>

<details>
  <summary style="font-size:1.25rem;">manually</summary>

1. Download the plugin

```sh
cd ~/git-repos  # ...or wherever you keep your Git repos/Zsh plugins
git clone https://github.com/Maneren/zsh-interactive-cd
```

2. Download binary

```sh
make download
```

or compile

```sh
make build
```

3. And then source it from `.zshrc`

```zsh
source ~/git-repos/zsh-interactive-cd/zsh-interactive-cd.plugin.zsh
```

</details>

### 2. Setup

And at the bottom of `.zshrc` bind the completion to shortcut

```zsh
zic-setup '^I'
```
