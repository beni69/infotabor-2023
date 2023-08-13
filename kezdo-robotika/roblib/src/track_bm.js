import { Robot } from "@kareszklub/roblib-client-node"

async function main() {
    const robot = await Robot.connect("localhost:1110");
    const run = true;

    let sp1 = 0.1
    let sp2 = 0.1
    let sp = 0.2
    let last = [true, true, true, true]
    await robot.drive(0.1, 0.1)

    while (run) {
        let track = await robot.trackSensor()
        if (track[0] && track[1] && track[2] && track[3]) {
            continue
        }

        sp1 = 0.08
        sp2 = 0.08
        if (!track[0]) {
            sp2 += 0.11
            sp1 -= 0.06
        }
        if (!track[1]) {
            sp2 += 0.07
        }
        if (!track[2]) {
            sp1 += 0.07
        }
        if (!track[3]) {
            sp2 -= 0.06
            sp1 += 0.11
        }


        console.log(sp1, sp2)
        await robot.drive(sp1, sp2)
    }

    await robot.stop()
    robot.disconnect()
}
main();
