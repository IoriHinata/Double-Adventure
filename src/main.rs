use macroquad::prelude::*;
use std::collections::HashMap;

const TILE:f32=25.0;
const CHUNK:i32=24;
const ACTIVE_RADIUS:i32=3;
const ZOOM_MIN:f32=0.55;
const ZOOM_MAX:f32=2.40;
const INK:Color=Color::new(0.16,0.12,0.08,1.0);
const PAPER:Color=Color::new(0.78,0.69,0.54,1.0);
const PANEL:Color=Color::new(0.87,0.78,0.63,0.97);
const GREEN:Color=Color::new(0.20,0.29,0.20,1.0);
const GREEN2:Color=Color::new(0.34,0.39,0.27,1.0);
const BLUE:Color=Color::new(0.29,0.39,0.47,1.0);
const RUST:Color=Color::new(0.57,0.25,0.14,1.0);
const PATH:Color=Color::new(0.43,0.32,0.20,1.0);

fn hash(x:i32,y:i32,salt:u32)->u32 {
    let mut n=(x as u32).wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263))
        .wrapping_add(salt.wrapping_mul(1442695041));
    n=(n^(n>>13)).wrapping_mul(1274126177);
    n^(n>>16)
}
fn river(x:i32,y:i32)->bool {
    let center=(x as f32*0.035).sin()*5.0+(x as f32*0.011).sin()*10.0;
    (y as f32-center).abs()<1.35
}
fn road(x:i32,y:i32)->bool {
    let horizontal=(y-(x.div_euclid(22)*22+4)).abs()<1 && x.rem_euclid(22)<20;
    let vertical=(x-(y.div_euclid(25)*25+6)).abs()<1 && y.rem_euclid(25)<22;
    horizontal||vertical
}
#[derive(Clone,Copy,PartialEq)]
enum Kind { Water,Tree,Rock,Grass,Path,House,Ruin,Fire,Crop,Bridge }
fn terrain(x:i32,y:i32)->Kind {
    if river(x,y) {
        if y.rem_euclid(19)==0 && x.rem_euclid(13)<5 {return Kind::Bridge;}
        return Kind::Water;
    }
    if road(x,y) {return Kind::Path;}
    let h=hash(x,y,31)%100;
    let grove=hash(x.div_euclid(5),y.div_euclid(5),8)%100;
    if grove>40 && h<73 {Kind::Tree}
    else if h<4 {Kind::Rock}
    else if h==5 {Kind::Fire}
    else if h==6 {Kind::House}
    else if h==7 {Kind::Ruin}
    else if h<14 {Kind::Crop}
    else {Kind::Grass}
}
#[derive(Clone,Copy)]
struct Tile { kind:Kind }
struct Chunk { tiles:Vec<Tile> }
impl Chunk {
    fn generate(cx:i32,cy:i32)->Self {
        let mut tiles=Vec::with_capacity((CHUNK*CHUNK) as usize);
        for ly in 0..CHUNK { for lx in 0..CHUNK {
            let wx=cx*CHUNK+lx;
            let wy=cy*CHUNK+ly;
            tiles.push(Tile{kind:terrain(wx,wy)});
        }}
        Self{tiles}
    }
    fn get(&self,lx:i32,ly:i32)->Tile {
        self.tiles[(ly*CHUNK+lx) as usize]
    }
}
struct World { chunks:HashMap<(i32,i32),Chunk> }
impl World {
    fn new()->Self {Self{chunks:HashMap::new()}}
    fn prepare(&mut self,px:i32,py:i32) {
        let cx=px.div_euclid(CHUNK);
        let cy=py.div_euclid(CHUNK);
        for oy in -ACTIVE_RADIUS..=ACTIVE_RADIUS {
            for ox in -ACTIVE_RADIUS..=ACTIVE_RADIUS {
                let key=(cx+ox,cy+oy);
                self.chunks.entry(key).or_insert_with(||Chunk::generate(key.0,key.1));
            }
        }
        self.chunks.retain(|(x,y),_|(x-cx).abs()<=ACTIVE_RADIUS && (y-cy).abs()<=ACTIVE_RADIUS);
    }
    fn tile(&self,x:i32,y:i32)->Tile {
        let cx=x.div_euclid(CHUNK);let cy=y.div_euclid(CHUNK);
        let lx=x.rem_euclid(CHUNK);let ly=y.rem_euclid(CHUNK);
        self.chunks.get(&(cx,cy)).map(|c|c.get(lx,ly)).unwrap_or(Tile{kind:terrain(x,y)})
    }
}
fn glyph(k:Kind,x:i32,y:i32)->(&'static str,Color) {
    match k {
        Kind::Water=>(if hash(x,y,2)%3==0{"~"}else{"≈"},BLUE),
        Kind::Tree=>(match hash(x,y,3)%5 {0=>"♣",1=>"Y",2=>"♠",3=>"T",_=>"♣"},GREEN),
        Kind::Rock=>(if hash(x,y,4)%2==0{"o"}else{"O"},INK),
        Kind::Path=>(if hash(x,y,5)%3==0{"·"}else{"."},PATH),
        Kind::House=>("⌂",RUST),Kind::Ruin=>("╫",INK),Kind::Fire=>("*",RUST),
        Kind::Crop=>(if hash(x,y,7)%2==0{"v"}else{"⁝"},GREEN2),
        Kind::Bridge=>("#",INK),
        Kind::Grass=>(match hash(x,y,6)%6 {0=>",",1=>"'",2=>".",3=>";",4=>"˙",_=>" "},GREEN2)
    }
}
fn walkable(k:Kind)->bool { !matches!(k,Kind::Water|Kind::Tree|Kind::Rock) }

struct Game {
    x:f32,y:f32,hp:i32,bag:i32,steps:u64,clock:f32,zoom:f32,
    panel:i32,facing:(i32,i32),message:String,message_time:f32,
    world:World,
}
impl Game {
    fn new()->Self {Self{x:127.0,y:84.0,hp:8,bag:0,steps:0,clock:8.0,zoom:1.0,panel:0,
        facing:(0,1),message:"A quiet morning in the Paper Wilds.".to_string(),message_time:4.0,world:World::new()}}
    fn move_by(&mut self,dx:f32,dy:f32,dt:f32) {
        if dx==0.0 && dy==0.0{return;}
        let len=(dx*dx+dy*dy).sqrt();let sx=dx/len;let sy=dy/len;
        self.facing=(sx.signum() as i32,sy.signum() as i32);
        let speed=3.2*dt;
        let nx=self.x+sx*speed;let ny=self.y+sy*speed;
        if walkable(self.world.tile(nx.floor() as i32,self.y.floor() as i32).kind){self.x=nx;}
        if walkable(self.world.tile(self.x.floor() as i32,ny.floor() as i32).kind){self.y=ny;}
        self.steps+=1;
    }
    fn interact(&mut self) {
        let tx=self.x.floor() as i32+self.facing.0;
        let ty=self.y.floor() as i32+self.facing.1;
        self.message=match self.world.tile(tx,ty).kind {
            Kind::Rock=>{self.bag+=1;"You found a smooth river stone.".to_string()},
            Kind::Crop=>{self.bag+=1;"You gathered wild grain.".to_string()},
            Kind::Fire=>{self.hp=8;"You rest by the fire. Health restored.".to_string()},
            Kind::House=>"An old field house. Its door is weathered.".to_string(),
            Kind::Ruin=>"A faded inscription: KEEP WALKING.".to_string(),
            Kind::Water=>"Cold water. Find a bridge.".to_string(),
            Kind::Tree=>"The forest is thick with old trees.".to_string(),
            _=>"Only wind moves across the paper.".to_string()
        };
        self.message_time=3.0;
    }
}
fn draw_paper() {
    clear_background(PAPER);
    let w=screen_width();let h=screen_height();
    for i in 0..(h/5.0) as i32 {
        let y=i as f32*5.0;
        draw_line(0.0,y,w,y,0.45,Color::new(0.35,0.24,0.14,0.09));
    }
    for i in 0..(w*h/7000.0) as i32 {
        let x=hash(i,17,3) as f32/u32::MAX as f32*w;
        let y=hash(i,29,4) as f32/u32::MAX as f32*h;
        draw_circle(x,y,0.55,Color::new(0.22,0.16,0.10,0.12));
    }
}
fn world_camera(g:&Game)->Camera2D {
    let scale=(screen_height()/720.0).clamp(0.70,1.30)*g.zoom;
    Camera2D{target:vec2((g.x+0.5)*TILE,(g.y+0.5)*TILE),
        zoom:vec2(2.0/screen_width(),-2.0/screen_height())*TILE*scale,..Default::default()}
}
fn draw_panel(x:f32,y:f32,w:f32,h:f32) {
    draw_rectangle(x,y,w,h,PANEL);
    draw_rectangle_lines(x,y,w,h,1.0,INK);
}
fn text(s:&str,x:f32,y:f32,size:f32) {draw_text(s,x,y,size,INK);}
fn button(s:&str,x:f32,y:f32,w:f32,h:f32)->bool {
    let (mx,my)=mouse_position();
    let inside=mx>=x&&mx<=x+w&&my>=y&&my<=y+h;
    draw_rectangle(x,y,w,h,if inside{Color::new(0.73,0.63,0.47,1.0)}else{PANEL});
    draw_rectangle_lines(x,y,w,h,1.0,INK);
    let fs=if h<34.0{14.0}else{17.0};
    let tw=measure_text(s,None,fs as u16,1.0).width;
    text(s,x+(w-tw)/2.0,y+h*0.68,fs);
    inside&&is_mouse_button_pressed(MouseButton::Left)
}
fn held(x:f32,y:f32,w:f32,h:f32)->bool {
    let (mx,my)=mouse_position();
    is_mouse_button_down(MouseButton::Left)&&mx>=x&&mx<=x+w&&my>=y&&my<=y+h
}
fn draw_hud(g:&Game) {
    let w=screen_width();let h=screen_height();
    let compact=w<620.0;let margin=if compact{8.0}else{16.0};
    let fs=if compact{12.0}else{17.0};
    let pw=if compact{205.0}else{280.0};let ph=if compact{77.0}else{101.0};
    draw_panel(margin,margin,pw,ph);
    text(&format!("DAY 03 | {:02}:{:02}",g.clock as i32,(g.clock.fract()*60.0) as i32),margin+9.0,margin+fs+4.0,fs);
    text(&format!("HP [{}{}]","#".repeat(g.hp as usize),".".repeat((8-g.hp).max(0) as usize)),margin+9.0,margin+fs*2.25,fs);
    text(&format!("BAG:{}  X:{} Y:{}",g.bag,g.x.floor() as i32,g.y.floor() as i32),margin+9.0,margin+fs*3.5,fs);
    let cx=w-margin-35.0;let cy=margin+24.0;
    text("N",cx,cy-8.0,fs);text("W + E",cx-15.0,cy+11.0,fs*0.72);text("S",cx,cy+27.0,fs);
    // Zoom controls are always in the top-right safe area.
    let zy=margin+65.0;
    if button("−",w-margin-76.0,zy,32.0,30.0){g.zoom=(g.zoom-0.12).max(ZOOM_MIN);}
    if button("+",w-margin-38.0,zy,32.0,30.0){g.zoom=(g.zoom+0.12).min(ZOOM_MAX);}
    // Bottom strip leaves the central play area unobstructed.
    let bar_h=if compact{34.0}else{42.0};let by=h-margin-bar_h;
    draw_panel(margin,by,(w-margin*2.0).min(310.0),bar_h);
    let help=if compact{"[E] USE   [I] BAG   [M] MAP   [Q] LOG"}else{"[E] INTERACT    [I] INVENTORY    [M] MAP    [Q] JOURNAL"};
    text(help,margin+8.0,by+bar_h*0.68,if compact{9.5}else{13.0});
    if g.message_time>0.0 {
        let fs=if compact{12.0}else{16.0};
        let maxw=(w-24.0).min(430.0);
        draw_panel((w-maxw)/2.0,ph+margin*2.0,maxw,29.0);
        text(&g.message,(w-maxw)/2.0+9.0,ph+margin*2.0+19.0,fs);
    }
}
fn draw_modal(g:&Game) {
    if g.panel==0{return;}
    let w=screen_width();let h=screen_height();
    draw_rectangle(0.0,0.0,w,h,Color::new(0.12,0.09,0.06,0.42));
    let pw=(w-32.0).min(420.0);let ph=220.0;
    let x=(w-pw)/2.0;let y=(h-ph)/2.0;
    draw_panel(x,y,pw,ph);
    text(match g.panel{1=>"BACKPACK",2=>"FIELD MAP",_=>"TRAVEL JOURNAL"},x+16.0,y+32.0,22.0);
    match g.panel {
        1=>{text(&format!("Collected supplies: {}",g.bag),x+16.0,y+76.0,17.0);text("Stone, grain and small discoveries.",x+16.0,y+104.0,15.0);}
        2=>{text("You are @ in the Paper Wilds.",x+16.0,y+76.0,16.0);text("Rivers, forests and old trails surround you.",x+16.0,y+104.0,15.0);}
        _=>{text("Every path leaves a mark on the page.",x+16.0,y+76.0,16.0);text(&format!("Steps travelled: {}",g.steps),x+16.0,y+104.0,16.0);}
    }
    if button("CLOSE",w/2.0-45.0,h/2.0+72.0,90.0,30.0){g.panel=0;}
}
fn draw_world(g:&Game) {
    let camera=world_camera(g);
    set_camera(&camera);
    let scale=(screen_height()/720.0).clamp(0.70,1.30)*g.zoom;
    let cols=(screen_width()/(TILE*scale)/2.0) as i32+3;
    let rows=(screen_height()/(TILE*scale)/2.0) as i32+3;
    let cx=g.x.floor() as i32;let cy=g.y.floor() as i32;
    for y in cy-rows..=cy+rows {
        for x in cx-cols..=cx+cols {
            let k=g.world.tile(x,y).kind;
            let (ch,color)=glyph(k,x,y);
            let px=x as f32*TILE;let py=y as f32*TILE;
            let size=if k==Kind::Tree{21.0}else{17.0};
            draw_text(ch,px+4.0,py+19.0,size,color);
            match k {
                Kind::House=>{draw_text("/\\",px-2.0,py+1.0,13.0,INK);draw_text("|_|",px+1.0,py+30.0,13.0,INK);}
                Kind::Ruin=>{draw_text("|||",px-2.0,py+30.0,11.0,INK);}
                Kind::Bridge=>{draw_line(px,py+8.0,px+TILE,py+8.0,1.0,INK);draw_line(px,py+17.0,px+TILE,py+17.0,1.0,INK);}
                _=>{}
            }
        }
    }
    draw_text("@",(g.x+0.5)*TILE-6.0,(g.y+0.5)*TILE+8.0,23.0,RUST);
    set_default_camera();
}
#[macroquad::main("PAPERLAND")]
async fn main() {
    let mut g=Game::new();
    loop {
        let dt=get_frame_time().min(0.05);
        g.clock+=dt/120.0;
        g.world.prepare(g.x.floor() as i32,g.y.floor() as i32);
        let mut dx=0.0;let mut dy=0.0;
        if is_key_down(KeyCode::A)||is_key_down(KeyCode::Left){dx-=1.0;}
        if is_key_down(KeyCode::D)||is_key_down(KeyCode::Right){dx+=1.0;}
        if is_key_down(KeyCode::W)||is_key_down(KeyCode::Up){dy-=1.0;}
        if is_key_down(KeyCode::S)||is_key_down(KeyCode::Down){dy+=1.0;}
        let w=screen_width();let h=screen_height();
        let compact=w<620.0;
        let dsize=if compact{38.0}else{48.0};
        let gap=3.0;
        let bx=w-(dsize*3.0+gap*2.0)-8.0;
        let by=h-(dsize*3.0+gap*2.0)-48.0;
        if held(bx+dsize+gap,by,dsize,dsize){dy-=1.0;}
        if held(bx,by+dsize+gap,dsize,dsize){dx-=1.0;}
        if held(bx+(dsize+gap)*2.0,by+dsize+gap,dsize,dsize){dx+=1.0;}
        if held(bx+dsize+gap,by+(dsize+gap)*2.0,dsize,dsize){dy+=1.0;}
        g.move_by(dx,dy,dt);
        if is_key_pressed(KeyCode::E){g.interact();}
        if is_key_pressed(KeyCode::I){g.panel=if g.panel==1{0}else{1};}
        if is_key_pressed(KeyCode::M){g.panel=if g.panel==2{0}else{2};}
        if is_key_pressed(KeyCode::Q){g.panel=if g.panel==3{0}else{3};}
        if is_key_pressed(KeyCode::Equal)||is_key_pressed(KeyCode::KpAdd){g.zoom=(g.zoom+0.12).min(ZOOM_MAX);}
        if is_key_pressed(KeyCode::Minus)||is_key_pressed(KeyCode::KpSubtract){g.zoom=(g.zoom-0.12).max(ZOOM_MIN);}
        if g.message_time>0.0{g.message_time-=dt;}
        draw_paper();
        draw_world(&g);
        draw_hud(&g);
        // Touch D-pad: held pointer gives continuous, frame-rate-independent movement.
        let dsize=if compact{38.0}else{48.0};
        let bx=w-(dsize*3.0+6.0)-8.0;
        let by=h-(dsize*3.0+6.0)-48.0;
        if button("▲",bx+dsize+3.0,by,dsize,dsize){}
        if button("◀",bx,by+dsize+3.0,dsize,dsize){}
        if button("▶",bx+(dsize+3.0)*2.0,by+dsize+3.0,dsize,dsize){}
        if button("▼",bx+dsize+3.0,by+(dsize+3.0)*2.0,dsize,dsize){}
        if button("E",w-48.0,by-35.0,40.0,30.0){g.interact();}
        let tab_y=h-42.0;
        if button("BAG",8.0,tab_y,47.0,28.0){g.panel=if g.panel==1{0}else{1};}
        if button("MAP",60.0,tab_y,47.0,28.0){g.panel=if g.panel==2{0}else{2};}
        if button("LOG",112.0,tab_y,47.0,28.0){g.panel=if g.panel==3{0}else{3};}
        draw_modal(&g);
        next_frame().await;
    }
}