# zsh-interactive-cd

## Demo

![demo](demo.gif)

## Installation

Requires `cargo` (install via [`rustup`](https://rustup.rs/))

### zinit

Add this snippet to `.zshrc`

```zsh
zinit ice make
zinit load Maneren/zsh-interactive-cd
```

### Oh-my-zsh

Download and compile the plugin

```sh
git clone https://github.com/Maneren/zsh-interactive-cd $ZSH_CUSTOM/plugins/zsh-interactive-cd
make -C $ZSH_CUSTOM/plugins/zsh-interactive-cd
```

And add `zsh-interactive-cd` to plugins list in `.zshrc`

```zsh
plugins=(
  ...

  zsh-interactive-cd
)
```

### Manually

Download and compile the plugin

```sh
cd ~/git-repos  # ...or wherever you keep your Git repos/Zsh plugins
git clone https://github.com/Maneren/zsh-interactive-cd
make -C zsh-interactive-cd
```

And then source it from `.zshrc`

```zsh
source ~/git-repos/zsh-interactive-cd/zsh-interactive-cd.plugin.zsh
```

## Usage

Press tab for completion as usual, it'll launch skim (fzf like interface). Check skim's readme for more search syntax usage.
