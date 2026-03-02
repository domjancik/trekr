@{
    # Local DirectShow video device on the development machine.
    DeviceName = "Cam Link 4K"
    # Optional stable DirectShow input selector, for example:
    # DeviceInput = "video=@device_pnp_\\?\usb#vid_0fd9&pid_0066&mi_00#...\\global"
    DeviceInput = ""

    # Cam Link 4K currently exposes 1920x1080@60 on this machine.
    VideoSize = "1920x1080"
    FrameRate = 60
    PixelFormat = "nv12"

    # Number of captured frames to skip before saving one image.
    # Increase this if the first frame after device open is unstable.
    SelectFrame = 0
}
