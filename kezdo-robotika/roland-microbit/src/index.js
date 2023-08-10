import { Robot, sleep } from "@kareszklub/roblib-client-node";
import { ReadlineParser, SerialPort } from "serialport"

async function main() {
    const robot = await Robot.connect("roland:1110");
    const serial = new SerialPort({ path: "/dev/ttyACM0", baudRate: 115_200, });
    const parser = serial.pipe(new ReadlineParser());

    parser.on("data", function(data) {
        const [left, right] = data.split(" ").map(Number);
        console.log([left, right]);
        robot.drive(left / 100, right / 100);
    });
}
main();
