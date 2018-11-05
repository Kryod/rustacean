command!(multiply(_ctx, msg, args) {
    let one = args.single::<i128>().unwrap();
    let two = args.single::<i128>().unwrap();

    let product = one * two;

    let _ = msg.channel_id.say(product);
});
