from time import sleep
import RPi.GPIO as GPIO

LED_R = 22
LED_G = 27
LED_B = 24

GPIO.setmode(GPIO.BCM)

GPIO.setup(TESZT, GPIO.OUT)

GPIO.output(LED_R, GPIO.HIGH)

sleep(1)

GPIO.cleanup()

