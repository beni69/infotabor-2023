import { SerialPort } from "serialport"

const serial = new SerialPort({
    path: "/dev/ttyACM0",
    baudRate: 125000,
});

serial.on("data", (data: any) => {
    process.stdout.write(data.toString())
})
