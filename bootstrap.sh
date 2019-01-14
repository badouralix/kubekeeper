#!/usr/bin/env bash

set -euo pipefail

DEFAULT_BINDIR=${KUBEKEEPER_BINDIR:-"$HOME/.bin"}
DEFAULT_CONFIGDIR=${KUBEKEEPER_CONFIGDIR:-"$HOME/.config/kubekeeper"}
DEFAULT_REPO="https://github.com/badouralix/kubekeeper.git"

usage() {
    cat << EOF
USAGE:
    bootstrap.sh                : install kubekeeper
    bootstrap.sh -d,--debug     : run in debug mode
    bootstrap.sh -f,--force     : erase all existing config files
    bootstrap.sh -h,--help      : show this message
    bootstrap.sh -l,--local     : install files from ./
    bootstrap.sh -s,--silent    : don't say anything, just do it
EOF
}

main() {
    # Set config
    FORCE=false
    LOCAL=false
    SILENT=false

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
        -h|--help)
            usage
            exit
            ;;
        -l|--local)
            LOCAL=true
            shift
            ;;
        -s|--silent)
            SILENT=true
            shift
            ;;
        *)
            echo "Unknown option $1"
            usage
            exit 1
        esac
    done

    # Get source
    if [[ "$LOCAL" == true ]]; then
        BASEDIR="."
    else
        BASEDIR=$(mktemp -d -t kubekeeper.XXXXXXXXXX)
        git clone $DEFAULT_REPO $BASEDIR
    fi

    # Install binaries
    if [[ "$SILENT" == true ]]; then
        BINDIR=$DEFAULT_BINDIR
    else
        read -p "Binary location? [$DEFAULT_BINDIR] " INPUT_BINDIR
        BINDIR=${INPUT_BINDIR:-$DEFAULT_BINDIR}
    fi

    mkdir -p $BINDIR

    install -m 0755 $BASEDIR/src/kubekeeper $BINDIR/kubekeeper

    # Install config files
    if [[ "$SILENT" == true ]]; then
        CONFIGDIR=$DEFAULT_CONFIGDIR
    else
        read -p "Config location? [$DEFAULT_CONFIGDIR] " CONFIGDIR
        CONFIGDIR=${CONFIGDIR:-$DEFAULT_CONFIGDIR}
    fi

    mkdir -p $CONFIGDIR

    if [[ -f $CONFIGDIR/exclude && "$FORCE" == true ]]; then
        [[ "$SILENT" == true ]] || echo "Overriding existing exclude config file."
        mv $CONFIGDIR/exclude $CONFIGDIR/exclude.bck
    fi
    if [[ -f $CONFIGDIR/exclude ]]; then
        [[ "$SILENT" == true ]] || echo "Exclude config file already exists, skipping."
    else
        cp $BASEDIR/config/exclude $CONFIGDIR/exclude
    fi

    if [[ -f $CONFIGDIR/include && "$FORCE" == true ]]; then
        [[ "$SILENT" == true ]] || echo "Overriding existing include config file."
        mv $CONFIGDIR/include $CONFIGDIR/include.bck
    fi
    if [[ -f $CONFIGDIR/include ]]; then
        [[ "$SILENT" == true ]] || echo "Include config file already exists, skipping."
    else
        cp $BASEDIR/config/include $CONFIGDIR/include
    fi

    # Export config
    if [[ "$SILENT" != true ]]; then
        echo
        echo "Thank you for installing kubekeeper."
        echo "Please add the following to your .zshrc:"
        echo
        echo -e "\talias kubectl=kubekeeper"
        echo -e "\tcompdef kubekeeper=kubectl"

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

main $@
