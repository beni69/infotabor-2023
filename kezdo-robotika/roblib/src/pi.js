import { Robot, sleep, EventType } from "@kareszklub/roblib-client-node";

const INP = 23, OUT = 24;

async function main() {
    const robot = await Robot.connect("beni-pi:1110");

    await robot.pinMode(INP, "input");
    await robot.pinMode(OUT, "output");

    function btnPressed(error, pressed) {
        if (error) {
            console.log("ERROR", error);
            return;
        }
        console.log(pressed);
        robot.writePin(OUT, pressed);
    }

    robot.subscribe(EventType.GpioPin, INP, btnPressed);

    await sleep(999999999999);

    robot.disconnect();
}
main();
