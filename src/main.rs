extern crate log;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    prelude::*,
    window::ReceivedCharacter,
};
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
        .add_resource(OngoingMusicDisplaySetting::default())
        .add_resource(OngoingMusicDisplayData::default())
        .add_startup_system(debug_spawn_ongoing_music.system())
        .add_system(print_char_event_system.system())
        .add_system(debug_log.system())
        .add_system(move_music_text_system.system())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PrintDiagnosticsPlugin::default())
        .run();
}

fn debug_spawn_ongoing_music(
    commands: &mut Commands,
    mut ongoing_music: ResMut<OngoingMusic>,
    mut fonts: ResMut<Fonts>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(UiCameraBundle::default());
    fonts.ongoing_music_font = Some(asset_server.load("fonts/FiraSans-Bold.ttf"));
    ongoing_music.0 = Some(PlayingMusic::new(TEST_SONG.to_string()));
    ongoing_music.spawn_text(commands, &*fonts);
}

#[derive(Default)]
struct Fonts {
    ongoing_music_font: Option<Handle<Font>>,
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
                            font: fonts
                                .ongoing_music_font
                                .clone()
                                .expect("tried to use unitialized music font"),
                            style: TextStyle {
                                font_size: 64.0,
                                color: Color::WHITE,
                                ..Default::default()
                            },
                        },
                        ..Default::default()
                    })
                    .with(MusicDisplayedLine(line_count));
            }
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

        let typed_from_lyric = TEST_SONG
            .chars()
            .take(playing_music.position_in_source_text())
            .collect::<String>();
        println!("{}", typed_from_lyric);
    }
}

struct OngoingMusicDisplaySetting {
    current_color: Color,
    non_current_color: Color,
    distance_between_line: f32,
    current_y: f32,
    top_displayed_line: usize,
    bottom_displayed_line: usize,
}

impl Default for OngoingMusicDisplaySetting {
    fn default() -> Self {
        Self {
            current_color: Color::RED,
            non_current_color: Color::WHITE,
            distance_between_line: 100.0,
            current_y: 1080.0 / 2.0, //TODO: compute at start and update on restart
            top_displayed_line: 3,
            bottom_displayed_line: 3,
        }
    }
}

struct OngoingMusicDisplayData {
    actual_y_coordinate: f32,
}

impl Default for OngoingMusicDisplayData {
    fn default() -> Self {
        Self {
            actual_y_coordinate: 0.0,
        }
    }
}

//TODO: only update what is required (put into multiple system and add an event ?)
fn move_music_text_system(
    ongoing_music: Res<OngoingMusic>,
    ongoing_music_setting: Res<OngoingMusicDisplaySetting>,
    mut ongoing_music_data: ResMut<OngoingMusicDisplayData>,
    mut query: Query<(&MusicDisplayedLine, &mut Style, &mut Text, &mut Draw)>,
) {
    if let Some(playing_music) = &(*ongoing_music).0 {
        let target_y_coordinate = ongoing_music_setting.current_y;
        ongoing_music_data.actual_y_coordinate = target_y_coordinate;
        let actual_line = playing_music.position_in_source_lines();
        for (MusicDisplayedLine(line_count), mut style, mut text, mut draw) in query.iter_mut() {
            let difference_isize: isize = *line_count as isize - actual_line as isize;
            if difference_isize > ongoing_music_setting.bottom_displayed_line as isize
                || difference_isize < -(ongoing_music_setting.top_displayed_line as isize)
            {
                draw.is_visible = false;
                continue;
            };
            draw.is_visible = true;
            let difference_f32 = difference_isize as f32;
            if *line_count == actual_line {
                text.style.color = ongoing_music_setting.current_color;
            } else {
                text.style.color = ongoing_music_setting.non_current_color;
            };
            style.position.top = Val::Px(
                difference_f32 * ongoing_music_setting.distance_between_line
                    + ongoing_music_data.actual_y_coordinate,
            );
        }
    }
}
