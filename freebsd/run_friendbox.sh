#!/bin/sh

cd /root/app

if [ -f /root/app/friendbox.env ]; then
    sed -i '' -e 's/\r$//' /root/app/friendbox.env 2>/dev/null || true
    . /root/app/friendbox.env
fi

export FRIENDBOX_BIND_ADDR
export DATABASE_URL

/usr/local/bin/friendbox
