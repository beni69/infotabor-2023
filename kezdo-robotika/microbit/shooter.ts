const BULLET_SPEED = 250;
const RADIO_GROUP = 215;

let pos = 2;

let bX: number = null, bY: number = null;
let ebX: number = null, ebY: number = null;

movePlayer(0);
function movePlayer(direction: number) {
    if (pos + direction < 0 || pos + direction > 4) {
        return;
    }

    led.unplot(pos, 4);
    pos += direction;
    led.plot(pos, 4);
}

function moveBullet() {
    if (bX !== null && bY !== null) {
        led.unplot(bX, bY)
    }
    bY--;
    led.plot(bX, bY);
}
function moveEnemyBullet() {
    if (ebX !== null && ebY !== null) {
        led.unplot(ebX, ebY)
    }
    ebY++;
    led.plot(ebX, ebY);
}

radio.setGroup(RADIO_GROUP);

input.onButtonPressed(Button.A, function () {
    movePlayer(-1);
})
input.onButtonPressed(Button.B, function () {
    movePlayer(1);
})
input.onButtonPressed(Button.AB, function () {
    bX = pos;
    bY = 3;
    led.plot(bX, bY);
    basic.pause(BULLET_SPEED);
    moveBullet();
    basic.pause(BULLET_SPEED);
    moveBullet();
    basic.pause(BULLET_SPEED);
    moveBullet();
    basic.pause(BULLET_SPEED);
    led.unplot(bX, bY);
    radio.sendNumber(bX);
    bX = null;
    bY = null;
})
radio.onReceivedNumber(function (n: number) {
    ebX = 4 - n;
    ebY = 0;
    led.plot(ebX, ebY);
    basic.pause(BULLET_SPEED);
    moveEnemyBullet();
    basic.pause(BULLET_SPEED);
    moveEnemyBullet();
    basic.pause(BULLET_SPEED);
    moveEnemyBullet();
    basic.pause(BULLET_SPEED);
    moveEnemyBullet();

    if (ebX === pos) {
        basic.showIcon(IconNames.Skull);
        radio.sendString("w");
    }
})
radio.onReceivedString(function (str: string) {
    if (str === "w") {
        basic.showIcon(IconNames.Yes);
    }
})
