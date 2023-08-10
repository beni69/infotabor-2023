radio.setGroup(123)
serial.redirectToUSB()

function clamp(n: number, min: number, max: number) {
    if (n < min) {
        return min
    }
    if (n > max) {
        return max
    }
    return n
}

basic.forever(function () {
    let p = clamp(input.rotation(Rotation.Pitch), -90, 90)
    let r = clamp(input.rotation(Rotation.Roll), -90, 90)
    if (Math.abs(p) == 90 || Math.abs(r) == 90) {
        return;
    }
    let speed = -p / (90 / 50);
    let rotation = 1 - Math.abs(r / 90);
    let left, right;
    if (r >= 0) {
        left = speed
        right = speed * rotation
    } else {
        right = speed
        left = speed * rotation
    }

    left = Math.roundWithPrecision(left, 2);
    right = Math.roundWithPrecision(right, 2);

    // serial.writeNumbers([left, right]);
    radio.sendString(left + " " + right);
})

