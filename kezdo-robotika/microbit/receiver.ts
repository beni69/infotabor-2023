radio.setGroup(123);
serial.redirectToUSB();
radio.onReceivedString(serial.writeLine)

