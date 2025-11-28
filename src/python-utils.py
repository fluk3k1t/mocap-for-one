import cv2_enumerate_cameras

def enumerate_cameras():
    camera_info = cv2_enumerate_cameras.enumerate_cameras()
    cameras = []
    for camera in camera_info:
        cameras.append((camera.index, camera.name))
    return cameras


__all__ = ['enumerate_cameras']