#!/bin/sh

function copy {
  profile=$1
  if [ -f target/$profile/servoshell ]
  then
    if [ -d target/$profile/ServoShell.app/Contents/MacOS ]
    then
      cp target/$profile/servoshell ./target/$profile/ServoShell.app/Contents/MacOS
    fi
  fi
}

copy debug
copy release
