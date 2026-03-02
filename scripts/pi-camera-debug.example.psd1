@{
    # Local DirectShow video device on the development machine.
    DeviceName = "usb video"
    # Optional stable DirectShow input selector, for example:
    # DeviceInput = "@device_pnp_\\?\usb#vid_534d&pid_2109&mi_00#...\\global"
    DeviceInput = ""

    # The currently connected HDMI capture device exposes 1920x1080@60 as MJPEG.
    VideoSize = "1920x1080"
    FrameRate = 60
    VideoCodec = "mjpeg"
    PixelFormat = ""

    # Number of captured frames to skip before saving one image.
    # Increase this if the first frame after device open is unstable.
    SelectFrame = 0
}
