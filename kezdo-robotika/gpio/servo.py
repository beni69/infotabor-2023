from time import sleep
import RPi.GPIO as GPIO

SERVO = 23

GPIO.setmode(GPIO.BCM)

GPIO.setup(SERVO, GPIO.OUT)

servo = GPIO.PWM(SERVO, 50)
servo.start(7.5)

#while True:
#	for n in range(50, 101, 5):
#		print(n)
#		servo.ChangeDutyCycle(n)
#		sleep(0.5)

sleep(1)

GPIO.cleanup()
