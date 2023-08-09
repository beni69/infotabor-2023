import { Robot, sleep, EventType } from "@kareszklub/roblib-client-node";

async function main() {
    const robot = await Robot.connect("roland:1110");

    // await robot.drive(.25, .25);
    // await sleep(1000);
    // await robot.drive(-.25, -.25);
    // await sleep(1000);
    // await robot.stop()

    while (true) {
        const distance = await robot.ultraSensor();
        console.log(distance);
        await sleep(100);
    }

    robot.disconnect();
}
main();
