import { Robot, sleep, EventType } from "@kareszklub/roblib-client-node";
import { dequal } from "dequal";

function clamp(n, min, max) {
    if (n < min) return min;
    if (n > max) return max;
    return n;
}

async function main() {
    const robot = await Robot.connect("roland:1110");

    process.on("SIGINT", async () => {
        await robot.stop();
        console.log("stoped");
        process.exit();
    })

    function onTrack(_err, track) {
        if (dequal(track, [true, true, true, true])) {
            console.log("track lost");
            return;
        }

        let left = 0, right = 0;
        if (!track[0]) {
            right += .2;
            // left -= .1;
        }
        if (!track[1])
            right += .2;
        if (!track[2])
            left += .2;
        if (!track[3]) {
            left += .2;
            // right -= .1;
        }

        if (!track[0] && left === 0) left -= .2;
        if (!track[3] && right === 0) right -= .2;

        // left = Math.min(left, .3)
        // right = Math.min(right, .3)
        left = clamp(left, -.3, .3);
        right = clamp(right, -.3, .3);

        console.log(track, [left, right]);
        robot.drive(left, right);
    }

    onTrack(null, await robot.trackSensor());

    robot.subscribe(EventType.TrackSensor, null, onTrack);

    await sleep(999999999999999);
    // robot.disconnect();
}
main();
