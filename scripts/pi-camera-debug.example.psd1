@{
    # Local DirectShow video device on the development machine.
    DeviceName = "Cam Link 4K"

    # Cam Link 4K currently exposes 1920x1080@60 on this machine.
    VideoSize = "1920x1080"
    FrameRate = 60
    PixelFormat = "nv12"

    # Number of captured frames to skip before saving one image.
    # Increase this if the first frame after device open is unstable.
    SelectFrame = 0
}
