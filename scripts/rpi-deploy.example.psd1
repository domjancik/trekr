@{
    Host = "192.168.1.247"
    User = "pi"
    Port = 22
    RemoteDir = "/home/pi/trekr"

    # Leave blank to use standard OpenSSH key/agent auth.
    SshKeyPath = ""

    # Optional. If set, deployment requires PuTTY's plink.exe and pscp.exe on PATH.
    Password = ""
}
