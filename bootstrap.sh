#!/usr/bin/env bash

set -euo pipefail

DEFAULT_BINDIR=${KUBEKEEPER_BINDIR:-"$HOME/.bin"}
DEFAULT_CONFIGDIR=${KUBEKEEPER_CONFIGDIR:-"$HOME/.config/kubekeeper"}
DEFAULT_REPO="https://github.com/badouralix/kubekeeper.git"

usage() {
    cat << EOF
USAGE:
    bootstrap.sh                : install kubekeeper
    bootstrap.sh uninstall      : uninstall kubekeeper

    bootstrap.sh -d,--debug     : run in debug mode
    bootstrap.sh -f,--force     : erase all existing config files
    bootstrap.sh -h,--help      : show this message
    bootstrap.sh -l,--local     : install files from ./
    bootstrap.sh -q,--quiet     : don't say anything, just do it
EOF
}

run_install() {
        # Get source
    if [[ "$LOCAL" == true ]]; then
        BASEDIR="."
    else
        BASEDIR=$(mktemp -d -t kubekeeper.XXXXXXXXXX)
        git clone -q $DEFAULT_REPO $BASEDIR
    fi

    # Install binaries
    if [[ "$QUIET" == true ]]; then
        BINDIR=$DEFAULT_BINDIR
    else
        read -p "Binary location? [$DEFAULT_BINDIR] " INPUT_BINDIR
        BINDIR=${INPUT_BINDIR:-$DEFAULT_BINDIR}
    fi

    mkdir -p $BINDIR

    install -m 0755 $BASEDIR/src/kubekeeper $BINDIR/kubekeeper

    # Install config files
    if [[ "$QUIET" == true ]]; then
        CONFIGDIR=$DEFAULT_CONFIGDIR
    else
        read -p "Config location? [$DEFAULT_CONFIGDIR] " CONFIGDIR
        CONFIGDIR=${CONFIGDIR:-$DEFAULT_CONFIGDIR}
    fi

    mkdir -p $CONFIGDIR

    if [[ -f $CONFIGDIR/exclude && "$FORCE" == true ]]; then
        [[ "$QUIET" == true ]] || echo "Overriding existing exclude config file."
        mv $CONFIGDIR/exclude $CONFIGDIR/exclude.bck
    fi
    if [[ -f $CONFIGDIR/exclude ]]; then
        [[ "$QUIET" == true ]] || echo "Exclude config file already exists, skipping."
    else
        cp $BASEDIR/config/exclude $CONFIGDIR/exclude
    fi

    if [[ -f $CONFIGDIR/include && "$FORCE" == true ]]; then
        [[ "$QUIET" == true ]] || echo "Overriding existing include config file."
        mv $CONFIGDIR/include $CONFIGDIR/include.bck
    fi
    if [[ -f $CONFIGDIR/include ]]; then
        [[ "$QUIET" == true ]] || echo "Include config file already exists, skipping."
    else
        cp $BASEDIR/config/include $CONFIGDIR/include
    fi

    # Export config
    if [[ "$QUIET" != true ]]; then
        echo
        echo "Thank you for installing kubekeeper."

        if [[ "$SHELL" =~ "/zsh"$ ]]; then
            echo "Please add the following to your .zshrc:"
        else
            echo "Please add the following to your rc file:"
        fi

        echo
        echo -e "\talias kubectl=kubekeeper"

        if [[ "$SHELL" =~ "/zsh"$ ]]; then
            echo -e "\tcompdef kubekeeper=kubectl"
        else
            echo -e "\tcomplete -o default -F __start_kubectl kubekeeper"
        fi

        if [[ ! ":$PATH:" =~ ":$BINDIR:" ]]; then
            echo -e "\texport PATH=$BINDIR:\$PATH"
        fi

        if [[ $CONFIGDIR != ${KUBEKEEPER_CONFIGDIR:-$DEFAULT_CONFIGDIR} ]]; then
            echo -e "\texport KUBEKEEPER_CONFIGDIR=$CONFIGDIR"
        fi

        echo
        echo "Enjoy Kubernetes, we're keeping you safe!"
    fi
}

run_uninstall() {
    # Configure verbose mode
    if [[ "$QUIET" == true ]]; then
        FLAG=""
    else
        FLAG="-v"
    fi

    # Delete binary
    rm $FLAG $DEFAULT_BINDIR/kubekeeper

    # Save config
    if [[ "$FORCE" == true ]]; then
        rm $FLAG $DEFAULT_CONFIGDIR/exclude
        rm $FLAG $DEFAULT_CONFIGDIR/include
    else
        mv $FLAG $DEFAULT_CONFIGDIR/exclude $DEFAULT_CONFIGDIR/exclude.bck
        mv $FLAG $DEFAULT_CONFIGDIR/include $DEFAULT_CONFIGDIR/include.bck
    fi
}

main() {
    # Set config
    FORCE=false
    LOCAL=false
    QUIET=false

    while (( $# ))
    do
        case $1 in
        -d|--debug)
            set -x
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -h|--help|help)
            usage
            exit
            ;;
        -l|--local)
            LOCAL=true
            shift
            ;;
        -q|--quiet)
            QUIET=true
            shift
            ;;
        install)
            run_install
            exit
            ;;
        uninstall)
            run_uninstall
            exit
            ;;
        *)
            echo "Unknown option $1"
            usage
            exit 1
        esac
    done

    # By default, install kubekeeper
    run_install

}

main $@
