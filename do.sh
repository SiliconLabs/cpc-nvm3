#!/bin/sh
set -eu

DIR=$(dirname "$0")

CMD=$1
shift

opt_fail() {
  case "$1" in
  "?") printf "unknown option -%s\n" "$2" ;;
  ":") printf "missing option argument -%s\n" "$2" ;;
  *) printf "unhandled option -%s, contact dev\n" "$1" ;;
  esac >&2
  exit 2
}

case "$CMD" in
build) #########################################################################

RUST_TARGET=host
CARGO_PROFILE=develop
while getopts ":t:r:" o; do
  case "$o" in
  t) RUST_TARGET=$OPTARG ;;
  r) CARGO_PROFILE=$OPTARG ;;
  *) opt_fail "$o" "$OPTARG" ;;
  esac
done

install -d ./target

case "$RUST_TARGET" in
host)
  RUST_TARGET=$(rustc -vV | grep ^host | awk '{print $2}')
  ln -sf "$RUST_TARGET" ./target/host
  ;;
esac

set -x
exec cargo --locked build --manifest-path "$DIR/Cargo.toml" --target-dir "./target" --target "$RUST_TARGET" --profile "$CARGO_PROFILE"

;; #############################################################################

install) #######################################################################

RUST_TARGET=host
CARGO_PROFILE=develop
DESTDIR=
PREFIX=/usr/local
INCLUDEDIR=include
LIBDIR=lib
while getopts ":t:r:d:p:i:l:" o; do
  case "$o" in
  t) RUST_TARGET=$OPTARG ;;
  r) CARGO_PROFILE=$OPTARG ;;
  d) DESTDIR=$OPTARG ;;
  p) PREFIX=$OPTARG ;;
  i) INCLUDEDIR=$OPTARG ;;
  l) LIBDIR=$OPTARG ;;
  *) opt_fail "$o" "$OPTARG" ;;
  esac
done

PROFILE_DIR="./target/$RUST_TARGET/$CARGO_PROFILE"
VERSION=$(grep "^  VERSION" "$DIR/CMakeLists.txt" | awk '{print $2}')

sed "s:@CMAKE_INSTALL_PREFIX@:$PREFIX:;s:@CMAKE_INSTALL_INCLUDEDIR@:$INCLUDEDIR:;s:@CMAKE_INSTALL_LIBDIR@:$LIBDIR:;s:@PROJECT_VERSION@:$VERSION:" <"$DIR/libcpc_nvm3.pc.in" >"$PROFILE_DIR/libcpc_nvm3.pc"

set -x

install -d "$DESTDIR$PREFIX/$INCLUDEDIR"
install -t "$DESTDIR$PREFIX/$INCLUDEDIR" "$PROFILE_DIR/cpc_nvm3.h"

install -d "$DESTDIR$PREFIX/$LIBDIR"
install -t "$DESTDIR$PREFIX/$LIBDIR" "$PROFILE_DIR/libcpc_nvm3.so"

install -d "$DESTDIR$PREFIX/$LIBDIR/pkgconfig"
install -t "$DESTDIR$PREFIX/$LIBDIR/pkgconfig" "$PROFILE_DIR/libcpc_nvm3.pc"

;; #############################################################################

*) printf "unknown command %s\n" "$CMD" >&2 && exit 2 ;;
esac
