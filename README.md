# kubekeeper

> Stop! Who would cross the Bridge of Death must answer me these questions three, ere the other side he see.
>
>> [The Bridgekeeper, _Monty Python and the Holy Grail_](https://www.youtube.com/watch?v=pWS8Mg-JWSg)

`kubekeeper` asks for context confirmation before running yolo `kubectl` commands on prod clusters. Just like `sudo`,
confirmation is required only if the command is run for the first time in a while.

## Requirements

Please install the following tools before using `kubekeeper`:

- [fzf](https://github.com/junegunn/fzf)
- [kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/)

## Installation

### Bootstrap script

Please **never** run `sh <(curl)` before reading!

```shell
sh <(curl -sL https://raw.githubusercontent.com/badouralix/kubekeeper/master/bootstrap.sh)
```

### Do it yourself

`kubekeeper` comes with a default configuration that fits all normal usages of `kubectl`. It can be installed by just
creating a [`kubekeeper` file](https://github.com/badouralix/kubekeeper/blob/master/src/kubekeeper) into your path:

```shell
# Download the latest version of kubekeeper
sudo curl -L https://raw.githubusercontent.com/badouralix/kubekeeper/master/src/kubekeeper -o /usr/local/bin/kubekeeper
# Apply executable permissions to the binary
sudo chmod +x /usr/local/bin/kubekeeper
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

### Whitelist and blacklist

Some contexts and/or commands may _never_ require validation. They are defined in the `exclude` config file.

Other contexts and/or commands may _always_ require validation. They are defined in the `include` config file.

### Script configuration

`kubekeeper` itself can be configured with the following environment variables:

| envvar | description | default |
|:------:|:----------- |:-------:|
| `KUBEKEEPER_CHECK_INTERVAL` | Number of seconds without execution before asking for confirmation again | 900 |
| `KUBEKEEPER_CONFIGDIR` | Configuration location | `~/.kube/kubekeeper` |
| `KUBEKEEPER_PIDFILE` | Pidfile name | `kubekeeper.pid` |

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
