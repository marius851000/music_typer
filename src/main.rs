use bevy::{prelude::*, window::ReceivedCharacter};
use music_typer::PlayingMusic;

static TEST_SONG: &str = r#"
Cutie Mark Crusaders, get out of my way
Those ponies need to know the truth
And they’ll hear it from me

Stop! Diamond Tiara, this is not the way
You know you’re better than this hostility

You don’t even know me at all
Don’t understand the meaning of my fall
What my family would think if I ever
Fail at anything

I’m a diamond – that means you’ll never break
No matter what be the cost of the path I take
Whatever I have to do to win in the end

Stop! This is not the answer
Wait! And it’s plainly seen
Listen! You can redeem yourself
But by helping others, not by being mean

We know you want friends who admire you
You want to be the star with all the power too
But there’s a better way, there’s a better wa-a-ay

There’s so much more still left to
Learn about yourself
See the light that shines in you
We know you can be somepony else

You can stop right now
And try another start
You’ll finally free yourself from the dark
And see the light
And see the light of your cutie mark
"#;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_system(print_char_event_system.system())
        .add_resource(OngoingMusic(Some(PlayingMusic::new(TEST_SONG.to_string()))))
        .add_system(debug_log.system())
        .run();
}

struct OngoingMusic(Option<PlayingMusic>);

#[derive(Default)]
struct State {
    event_reader: EventReader<ReceivedCharacter>,
}

fn print_char_event_system(
    mut ongoing_music: ResMut<OngoingMusic>,
    mut state: Local<State>,
    char_input_events: Res<Events<ReceivedCharacter>>,
) {
    for event in state.event_reader.iter(&char_input_events) {
        if let Some(music) = ongoing_music.0.as_mut() {
            music.add_typed_char(event.char);
            println!("{}", music.get_typed_text())
        };
    }
}

fn debug_log(ongoing_music: Res<OngoingMusic>) {
    if let Some(playing_music) = ongoing_music.0.as_ref() {
        println!("{:?}", playing_music.get_typed_text());
        println!("{:?}", playing_music.correctness());

        let mut typed_from_lyric = TEST_SONG.chars().take(playing_music.get_position_in_source_text()).collect::<String>();
        println!("{}", typed_from_lyric);
    }
}
