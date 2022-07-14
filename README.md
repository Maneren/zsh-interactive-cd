# zsh-interactive-cd

## Demo

![demo](demo.gif)

## Usage

Press TAB for completion as usual, it'll launch skim (fzf like interface). Check skim's readme for more search syntax usage.

## Configuration

Set enviroment variable with given name to `1` or `true`

| option | effect |
| --- | --- |
| `zic-case-insensitive` | ignores case |
| `zic-ignore-dots` | includes hidden files in all searches |

## Installation

Download from Gitub releases or compile with `cargo` (install that via [`rustup`](https://rustup.rs/))

### zinit

1. download binary

```zsh
zinit ice make'download'
zinit load Maneren/zsh-interactive-cd
```

1. or compile

```zsh
zinit ice make'build'
zinit load Maneren/zsh-interactive-cd
```

### Oh-my-zsh

Download the plugin

```sh
git clone https://github.com/Maneren/zsh-interactive-cd $ZSH_CUSTOM/plugins/zsh-interactive-cd
```

1. download binary

```sh
make download -C $ZSH_CUSTOM/plugins/zsh-interactive-cd
```

1. or compile

```sh
make build -C $ZSH_CUSTOM/plugins/zsh-interactive-cd
```

And add `zsh-interactive-cd` to plugins list in `.zshrc`

```zsh
plugins=(
  ...

  zsh-interactive-cd
)
```

### Manually

Download the plugin

```sh
cd ~/git-repos  # ...or wherever you keep your Git repos/Zsh plugins
git clone https://github.com/Maneren/zsh-interactive-cd
```

1. download binary

```sh
make download
```

1. or compile

```sh
make build
```

And then source it from `.zshrc`

```zsh
source ~/git-repos/zsh-interactive-cd/zsh-interactive-cd.plugin.zsh
```
