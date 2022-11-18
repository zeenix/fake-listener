Step to reproduce:

Start the producer, check the signal is sent with

```sh
busctl --user monitor ludo_ic.daemon.producer
```

Then start the listener with zbus logs.

Open a shell and send some dbus command to the listener (I'm not sure if it is really needed)
```sh
while [ 1 ]; do busctl --user call ludo_ic.daemon.other /ludo_ic/daemon/other ludo_ic.daemon.other SayHello; sleep 0.3; done
```

On my machine (i7-1165G7), after few minutes the red "error" log isn't printed anymore, that's the moment when the signal is received by zbus but the proxy does not receive it.
