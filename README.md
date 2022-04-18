# Kubekeeper

> Stop! Who would cross the Bridge of Death must answer me these questions three, ere the other side he see.
>
>> [The Bridgekeeper, _Monty Python and the Holy Grail_](https://www.youtube.com/watch?v=pWS8Mg-JWSg)

`kubekeeper` asks for context confirmation before running yolo `kubectl` commands on prod clusters. Just like `sudo`,
confirmation is required only if the command is run for the first time in a while.

## Installation

### Homebrew

```bash
brew install badouralix/tap/kubekeeper
```

### Do it yourself

Please install the following tools before using `kubekeeper`:

- [kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/)
- [rust and cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

`kubekeeper` comes with a default configuration that fits all normal usages of `kubectl`. It can be installed by just
creating a `kubekeeper` file into your path:

```shell
# Download the latest version of kubekeeper
git clone https://github.com/badouralix/kubekeeper.git

# Build kubekeeper
cd kubekeeper/
cargo build --release --bin kubekeeper

# Install kubekeeper
sudo install target/release/kubekeeper /usr/local/bin/kubekeeper
```

### Aliases and autocompletion

You may add the following lines to your `.zshrc` to use `kubekeeper` with all your existing aliases:

```shell
# kubekeeper completion must be added after kubectl completion

alias kubectl=kubekeeper
compdef kubekeeper=kubectl
```

Autocompletion is also available in other shells with:

```shell
# kubekeeper completion must be added after kubectl completion

complete -o default -F __start_kubectl kubekeeper
```

## Usage

`kubekeeper` is an invisible wrapper on top of `kubectl`, and must therefore be used just like the latter.

## Configuration

`kubekeeper` itself can be configured with the following environment variables:

|    Environment Variable     | Description                                                              |     Default      |
| :-------------------------: | :----------------------------------------------------------------------- | :--------------: |
| `KUBEKEEPER_CHECK_INTERVAL` | Number of seconds without execution before asking for confirmation again |       900        |
|    `KUBEKEEPER_PIDFILE`     | Pidfile name                                                             | `kubekeeper.pid` |

Note that the pidfile is always located in the user temp folder (usually `/tmp` on Linux, and
`/var/folders/xx/xxx..xxx/T` on macOS).

## Troubleshooting

### Completion with [`kubectl-fzf`](https://github.com/bonnefoa/kubectl-fzf) shows files and dirs

`kubekeeper` integrates well with [`kubectl-fzf`](https://github.com/bonnefoa/kubectl-fzf). Make sure to define
completion in the right order:

```shell
source <(kubectl completion $SHELL)
source $GOPATH/src/github.com/bonnefoa/kubectl-fzf/kubectl_fzf.sh
complete -o default -F __start_kubectl kubekeeper
```

## License

Unless expressly stated otherwise, all contents licensed under the [MIT License](LICENSE).
