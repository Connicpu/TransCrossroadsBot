#!/bin/sh
REMOTE="connie@wf.ukl.me"
REMOTE_DIR="/home/connie/transybot"
scp target/release/transcrossroadsbot "$REMOTE:$REMOTE_DIR/transybot-tmp"
ssh $REMOTE "
    cd $REMOTE_DIR
    rm transybot
    mv transybot-tmp transybot
    chmod 700 transybot
    ls -al
    echo 'Restarting Transybot'
    systemctl --user restart transybot
"
