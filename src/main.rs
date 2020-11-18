#[macro_use]
extern crate log;

use bevy::{prelude::*, window::ReceivedCharacter, log::LogPlugin};
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
    env_logger::init();

    App::build()
        .add_plugins(DefaultPlugins)
        .add_resource(OngoingMusic(None))
        .add_resource(Fonts::default())
        .add_startup_system(debug_spawn_ongoing_music.system())
        .add_system(print_char_event_system.system())
        .add_system(debug_log.system())
        .add_system(move_music_text_system.system())
        .run();
}

fn debug_spawn_ongoing_music(
    commands: &mut Commands,
    mut ongoing_music: ResMut<OngoingMusic>,
    mut fonts: ResMut<Fonts>,
    asset_server: Res<AssetServer>,
) {

    commands
        .spawn(UiCameraBundle::default());
    fonts.ongoing_music_font = Some(asset_server.load("fonts/FiraSans-Bold.ttf"));
    ongoing_music.0 = Some(PlayingMusic::new(TEST_SONG.to_string()));
    ongoing_music.spawn_text(commands, &*fonts);
}

#[derive(Default)]
struct Fonts {
    ongoing_music_font: Option<Handle<Font>>
}

struct OngoingMusic(Option<PlayingMusic>);

/// a line of a song, the usize it the line number
struct MusicDisplayedLine(usize);

//TODO: move this to a system that does stuff at OngoingMusic instanciation (also, automatically delete at uninstantiation)
impl OngoingMusic {
    fn spawn_text(&self, commands: &mut Commands, fonts: &Fonts) {
        if let Some(playing_music) = self.0.as_ref() {
            for (line_count, line) in playing_music.lines().iter().enumerate() {
                commands
                    .spawn(TextBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            ..Default::default()
                        },
                        text: Text {
                            value: line.to_string(),
                            font: fonts.ongoing_music_font.clone().expect("tried to use unitialized music font"),
                            style: TextStyle {
                                font_size: 64.0,
                                color: Color::WHITE,
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    })
                    .with(MusicDisplayedLine(line_count));
            };
        };
    }
}

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
        };
    }
}

fn debug_log(ongoing_music: Res<OngoingMusic>) {
    if let Some(playing_music) = ongoing_music.0.as_ref() {
        println!("{:?}", playing_music.get_typed_text());
        println!("{:?}", playing_music.correctness());

        let typed_from_lyric = TEST_SONG.chars().take(playing_music.position_in_source_text()).collect::<String>();
        println!("{}", typed_from_lyric);
    }
}

fn move_music_text_system(
    ongoing_music: Res<OngoingMusic>,
    mut query: Query<(&MusicDisplayedLine, &mut Style)>
) {
    if let Some(playing_music) = &(*ongoing_music).0 {
        let actual_line = playing_music.position_in_source_lines() as f32;
        for (MusicDisplayedLine(line_count), mut style) in query.iter_mut() {
            let difference = *line_count as f32 - actual_line;
            style.position.top = Val::Px(difference * 100.0);
        }
    }
}
