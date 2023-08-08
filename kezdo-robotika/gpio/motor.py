from time import sleep
import RPi.GPIO as GPIO

IN1 = 20 # bal előre
IN2 = 21 # bal hátra
IN3 = 19 # jobb előre
IN4 = 26 # jobb hátra
ENA = 16 # bal sebesség (pwm)
ENB = 13 # jobb sebesség (pwm)

GPIO.setmode(GPIO.BCM)
GPIO.setup(ENA, GPIO.OUT, initial = GPIO.HIGH)
GPIO.setup(IN1, GPIO.OUT, initial = GPIO.LOW)
GPIO.setup(IN2, GPIO.OUT, initial = GPIO.LOW)
GPIO.setup(ENB, GPIO.OUT, initial = GPIO.HIGH)
GPIO.setup(IN3, GPIO.OUT, initial = GPIO.LOW)
GPIO.setup(IN4, GPIO.OUT, initial = GPIO.LOW)
pwm_L = GPIO.PWM(ENA, 2000)
pwm_R = GPIO.PWM(ENB, 2000)

pwm_L.start(25)
pwm_R.start(25)
GPIO.output(IN1, GPIO.HIGH)
GPIO.output(IN3, GPIO.HIGH)

sleep(1)

GPIO.output(IN1, GPIO.LOW)
GPIO.output(IN3, GPIO.LOW)
GPIO.output(IN2, GPIO.HIGH)
GPIO.output(IN4, GPIO.HIGH)

sleep(1)

GPIO.output(IN2, GPIO.LOW)
GPIO.output(IN4, GPIO.LOW)

GPIO.cleanup()
