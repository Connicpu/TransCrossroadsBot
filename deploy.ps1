#!/bin/sh
$REMOTE="connie@wf.ukl.me"
$REMOTE_DIR="/home/connie/transybot"
scp .env "$REMOTE`:$REMOTE_DIR/.env"
scp target/release/transcrossroadsbot "$REMOTE`:$REMOTE_DIR/transybot-tmp"
ssh $REMOTE "
    cd $REMOTE_DIR
    if [ -f transybot-tmp ]; then
        rm transybot
        mv transybot-tmp transybot
        chmod 700 transybot
        ls -al
    else
        echo 'New executable not found'
    fi
    echo 'Restarting Transybot'
    systemctl --user restart transybot
"
