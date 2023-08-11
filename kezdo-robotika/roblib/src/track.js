import { Robot, sleep, EventType } from "@kareszklub/roblib-client-node";
import { dequal } from "dequal";

async function main() {
    const robot = await Robot.connect("roland:1110");

    function onTrack(_err, track) {
        if (dequal(track, [true, false, false, true])) {
            console.log(true, track);
            robot.drive(.2, .2);
        } else {
            console.log(false, track);
            robot.stop();
        }
    }
    robot.subscribe(EventType.TrackSensor, null, onTrack);

    onTrack(null, await robot.trackSensor());

    await sleep(999999999999999);

    robot.disconnect();
}
main();
