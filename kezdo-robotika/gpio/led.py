from time import sleep
import RPi.GPIO as GPIO

LED_R = 22
LED_G = 27
LED_B = 24

GPIO.setmode(GPIO.BCM)

GPIO.setup(LED_R, GPIO.OUT)
#GPIO.setup(LED_G, GPIO.OUT)
GPIO.setup(LED_B, GPIO.OUT)

GPIO.output(LED_R, GPIO.HIGH)
GPIO.output(LED_G, GPIO.HIGH)
GPIO.output(LED_B, GPIO.HIGH)

sleep(1)

GPIO.cleanup()
