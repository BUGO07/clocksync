# Synchronise your system clock to a time server

## Usage

Note - the arguments are not necessary, the timeserver defaults to time.windows.com and the timezone defaults to UTC+0. for timezones only use UTC+time.

### Linux

sudo ./clocksync-linux-x86-64 --server \<timeserver> --zone \<timezone>

### Windows

You need to run this with administrator permissions.

clocksync-windows-x86-64.exe --server \<timeserver> --zone \<timezone>

## Building

### Requirements

rustc, cargo

### Build Command

cargo build --release
