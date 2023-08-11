import { Robot, sleep, EventType } from "@kareszklub/roblib-client-node";

async function main() {
    const robot = await Robot.connect("roland:1110");

    process.stdin.setRawMode(true);
    process.stdin.resume();
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", async function(key) {
        // Ctrl-C kezelése -> program vége
        if (key === "\u0003") {
            robot.disconnect();
            process.exit();
        }

        if (key === "w") {
            console.log("move")
        } else if (key === "s") {
            console.log("stop")
        }
    });
}
main();
