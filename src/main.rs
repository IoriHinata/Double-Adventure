use macroquad::prelude::*;

const TILE: f32 = 30.0;
const INK: Color = Color::new(0.18, 0.14, 0.10, 1.0);
const DARK: Color = Color::new(0.12, 0.18, 0.13, 1.0);
const WATER: Color = Color::new(0.31, 0.39, 0.43, 1.0);
const PAPER: Color = Color::new(0.88, 0.80, 0.65, 1.0);
const PANEL: Color = Color::new(0.91, 0.84, 0.71, 0.96);

#[derive(Clone, Copy, PartialEq)]
enum Kind { Grass, Forest, Water, Rock, Cabin, Ruin, Camp, Path, Field }

fn hash(x: i32, y: i32, seed: u32) -> u32 {
    let mut n = (x as u32).wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263))
        .wrapping_add(seed.wrapping_mul(1442695041));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    n ^ (n >> 16)
}
fn smooth_noise(x: i32, y: i32, scale: i32, seed: u32) -> f32 {
    let gx = x.div_euclid(scale); let gy = y.div_euclid(scale);
    let fx = x.rem_euclid(scale) as f32 / scale as f32;
    let fy = y.rem_euclid(scale) as f32 / scale as f32;
    let a = hash(gx, gy, seed) as f32 / u32::MAX as f32;
    let b = hash(gx+1, gy, seed) as f32 / u32::MAX as f32;
    let c = hash(gx, gy+1, seed) as f32 / u32::MAX as f32;
    let d = hash(gx+1, gy+1, seed) as f32 / u32::MAX as f32;
    let sx = fx*fx*(3.0-2.0*fx); let sy = fy*fy*(3.0-2.0*fy);
    (a+(b-a)*sx)*(1.0-sy)+(c+(d-c)*sx)*sy
}
fn kind(x: i32, y: i32) -> Kind {
    let n = smooth_noise(x,y,13,7);
    let detail = hash(x,y,11) % 100;
    if n < 0.22 { Kind::Water }
    else if n > 0.78 && detail < 72 { Kind::Forest }
    else if detail < 3 { Kind::Rock }
    else if detail < 5 { Kind::Camp }
    else if detail < 7 { Kind::Ruin }
    else if detail < 10 { Kind::Cabin }
    else if detail < 13 { Kind::Field }
    else { Kind::Grass }
}
fn glyph(k: Kind, x: i32, y: i32) -> &'static str {
    match k {
        Kind::Water => if hash(x,y,2)%3==0 {"~"} else {"≈"},
        Kind::Forest => match hash(x,y,3)%4 {0=>"♣",1=>"♠",2=>"#",_=>"♣"},
        Kind::Rock => if hash(x,y,4)%2==0 {"o"} else {"O"},
        Kind::Cabin => "+",
        Kind::Ruin => "⌂",
        Kind::Camp => "*",
        Kind::Path => ".",
        Kind::Field => if hash(x,y,5)%2==0 {"v"} else {"'"},
        Kind::Grass => match hash(x,y,6)%5 {0=>",",1=>"'",2=>".",3=>":",_=>" "},
    }
}
fn walkable(k: Kind) -> bool { !matches!(k, Kind::Water | Kind::Forest | Kind::Rock) }

struct Game {
    px: f32, py: f32, facing: (f32,f32), hp: i32, steps: u64,
    inventory: i32, show_map: bool, show_inventory: bool, journal: bool,
    seed: u32, message: String, message_time: f32,
}
impl Game {
    fn new() -> Self { Self { px:0.0,py:0.0,facing:(0.0,1.0),hp:10,steps:0,inventory:0,
        show_map:false,show_inventory:false,journal:false,seed:7319,message:"THE PAPER WILDS — find your own path".into(),message_time:5.0 } }
    fn move_by(&mut self, dx:f32,dy:f32) {
        let nx=self.px+dx; let ny=self.py+dy;
        let tx=nx.floor() as i32; let ty=ny.floor() as i32;
        if walkable(kind(tx,ty)) { self.px=nx; self.py=ny; self.steps+=1; }
        self.facing=(dx,dy);
    }
    fn interact(&mut self) {
        let x=(self.px+self.facing.0.signum()).floor() as i32;
        let y=(self.py+self.facing.1.signum()).floor() as i32;
        match kind(x,y) {
            Kind::Rock => { self.inventory+=1; self.message="You pocket a smooth stone.".into(); }
            Kind::Camp => { self.hp=10; self.message="A quiet rest. Health restored.".into(); }
            Kind::Cabin => self.message="An empty cabin. Someone was here.".into(),
            Kind::Ruin => self.message="Letters on the wall: KEEP WALKING.".into(),
            Kind::Field => { self.inventory+=1; self.message="You gather a handful of wild grain.".into(); }
            Kind::Water => self.message="The river is too deep to cross.".into(),
            Kind::Forest => self.message="The thicket is too dense.".into(),
            _ => self.message="Only wind and paper rustle here.".into(),
        }
        self.message_time=3.5;
    }
}
fn draw_paper() {
    clear_background(PAPER);
    let w=screen_width(); let h=screen_height();
    for i in 0..((h/7.0) as i32+1) {
        let y=i as f32*7.0;
        draw_line(0.0,y,w,y,0.45,Color::new(0.42,0.31,0.20,0.10));
    }
    for i in 0..((w*h/19000.0) as i32) {
        let x=((i*97+31) as f32*17.13).rem_euclid(w);
        let y=((i*53+11) as f32*13.71).rem_euclid(h);
        draw_circle(x,y,0.55,Color::new(0.28,0.20,0.12,0.13));
    }
}
fn button(label:&str,x:f32,y:f32,w:f32,h:f32) -> bool {
    let (mx,my)=mouse_position();
    let hover=mx>=x&&mx<=x+w&&my>=y&&my<=y+h;
    draw_rectangle(x,y,w,h,if hover {Color::new(0.75,0.66,0.51,0.98)} else {PANEL});
    draw_rectangle_lines(x,y,w,h,1.2,INK);
    let s= if label.len()>5 {17.0} else {22.0};
    let tw=measure_text(label,None,s as u16,1.0).width;
    draw_text(label,x+(w-tw)/2.0,y+h*0.68,s,INK);
    hover && is_mouse_button_pressed(MouseButton::Left)
}
fn main_camera(g:&Game) -> Camera2D {
    let scale=1.0;
    Camera2D { target:vec2(g.px*TILE,g.py*TILE), zoom:vec2(2.0/screen_width(),-2.0/screen_height())*TILE*scale, ..Default::default() }
}
#[macroquad::main("PAPERLAND")]
async fn main() {
    let mut g=Game::new();
    loop {
        let dt=get_frame_time().min(0.05);
        let mut dx=0.0; let mut dy=0.0;
        if is_key_down(KeyCode::A)||is_key_down(KeyCode::Left){dx-=1.0;}
        if is_key_down(KeyCode::D)||is_key_down(KeyCode::Right){dx+=1.0;}
        if is_key_down(KeyCode::W)||is_key_down(KeyCode::Up){dy-=1.0;}
        if is_key_down(KeyCode::S)||is_key_down(KeyCode::Down){dy+=1.0;}
        let speed=if is_key_down(KeyCode::LeftShift){5.0}else{3.1};
        if dx!=0.0||dy!=0.0 {
            let l=(dx*dx+dy*dy).sqrt(); g.move_by(dx/l*speed*dt,dy/l*speed*dt);
        }
        if is_key_pressed(KeyCode::E){g.interact();}
        if is_key_pressed(KeyCode::I){g.show_inventory=!g.show_inventory;}
        if is_key_pressed(KeyCode::M){g.show_map=!g.show_map;}
        if is_key_pressed(KeyCode::Q){g.journal=!g.journal;}
        if g.message_time>0.0 {g.message_time-=dt;}

        draw_paper();
        set_camera(&main_camera(&g));
        let cx=g.px.floor() as i32; let cy=g.py.floor() as i32;
        let cols=(screen_width()/TILE/2.0) as i32+3;
        let rows=(screen_height()/TILE/2.0) as i32+3;
        for y in cy-rows..=cy+rows { for x in cx-cols..=cx+cols {
            let k=kind(x,y);
            let px=x as f32*TILE; let py=y as f32*TILE;
            let glyph=glyph(k,x,y);
            let color=match k {Kind::Water=>WATER,Kind::Forest=>DARK,Kind::Rock=>INK,Kind::Camp=>Color::new(0.55,0.22,0.12,1.0),_=>Color::new(0.27,0.22,0.16,0.8)};
            let size=match k {Kind::Forest=>25.0,Kind::Cabin|Kind::Ruin=>26.0,_=>20.0};
            draw_text(glyph,px+4.0,py+23.0,size,color);
            if matches!(k,Kind::Cabin) {
                draw_text("[-|-]",px-9.0,py-2.0,12.0,INK);
                draw_text("|___|",px-9.0,py+32.0,12.0,INK);
            }
            if matches!(k,Kind::Ruin) {draw_text("/###\\",px-8.0,py-4.0,12.0,INK);}
        }}
        draw_text("@",g.px*TILE+3.0,g.py*TILE+23.0,25.0,Color::new(0.55,0.16,0.10,1.0));
        set_default_camera();

        // Paper HUD, typewriter labels and coordinates
        draw_rectangle(12.0,10.0,250.0,88.0,PANEL);
        draw_rectangle_lines(12.0,10.0,250.0,88.0,1.5,INK);
        draw_text("DAY 03 | THE PAPER WILDS",22.0,33.0,17.0,INK);
        draw_text(&format!("HP [{}]", "#".repeat(g.hp as usize)),22.0,55.0,18.0,INK);
        draw_text(&format!("BAG: {}   X:{} Y:{}",g.inventory,g.px.floor() as i32,g.py.floor() as i32),22.0,80.0,15.0,INK);
        draw_text("N",screen_width()-28.0,28.0,17.0,INK);
        draw_text("W + E",screen_width()-58.0,47.0,13.0,INK);
        draw_text("S",screen_width()-28.0,64.0,17.0,INK);

        let bottom=screen_height()-48.0;
        draw_rectangle(12.0,bottom-7.0,screen_width()-24.0,42.0,PANEL);
        draw_rectangle_lines(12.0,bottom-7.0,screen_width()-24.0,42.0,1.2,INK);
        draw_text("[E] INTERACT     [I] BAG     [M] MAP     [Q] JOURNAL",22.0,bottom+20.0,15.0,INK);
        if g.message_time>0.0 {
            let width=(measure_text(&g.message,None,18,1.0).width+28.0).min(screen_width()-30.0);
            draw_rectangle((screen_width()-width)/2.0,108.0,width,34.0,PANEL);
            draw_rectangle_lines((screen_width()-width)/2.0,108.0,width,34.0,1.0,INK);
            draw_text(&g.message,(screen_width()-width)/2.0+12.0,131.0,18.0,INK);
        }

        // Touch controls, sized for phones
        let bx=screen_width()-142.0; let by=screen_height()-190.0;
        if button("^",bx+44.0,by,42.0,40.0){g.move_by(0.0,-0.7);}
        if button("<",bx,by+43.0,42.0,40.0){g.move_by(-0.7,0.0);}
        if button(">",bx+88.0,by+43.0,42.0,40.0){g.move_by(0.7,0.0);}
        if button("v",bx+44.0,by+86.0,42.0,40.0){g.move_by(0.0,0.7);}
        if button("E",screen_width()-82.0,by-48.0,42.0,38.0){g.interact();}
        if button("BAG",12.0,screen_height()-104.0,58.0,34.0){g.show_inventory=!g.show_inventory;}
        if button("MAP",76.0,screen_height()-104.0,58.0,34.0){g.show_map=!g.show_map;}
        if button("LOG",140.0,screen_height()-104.0,58.0,34.0){g.journal=!g.journal;}

        if g.show_inventory || g.show_map || g.journal {
            let title=if g.show_inventory {"BACKPACK"} else if g.show_map {"FIELD NOTES / MAP"} else {"TRAVEL JOURNAL"};
            let x=screen_width()*0.5-145.0; let y=screen_height()*0.5-95.0;
            draw_rectangle(x,y,290.0,190.0,PANEL); draw_rectangle_lines(x,y,290.0,190.0,2.0,INK);
            draw_text(title,x+18.0,y+30.0,22.0,INK);
            if g.show_inventory {draw_text(&format!("Collected objects: {}",g.inventory),x+18.0,y+68.0,17.0,INK);draw_text("A stone, a seed, a story.",x+18.0,y+96.0,16.0,INK);}
            else if g.show_map {draw_text("You are the @ on an endless",x+18.0,y+68.0,16.0,INK);draw_text("procedural sheet of paper.",x+18.0,y+92.0,16.0,INK);draw_text("Explore to reveal more land.",x+18.0,y+116.0,16.0,INK);}
            else {draw_text("Every direction is a new page.",x+18.0,y+68.0,16.0,INK);draw_text(&format!("Steps taken: {}",g.steps),x+18.0,y+96.0,16.0,INK);}
            if button("CLOSE",x+100.0,y+140.0,90.0,34.0){g.show_inventory=false;g.show_map=false;g.journal=false;}
        }
        next_frame().await;
    }
}
