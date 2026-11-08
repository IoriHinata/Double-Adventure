use macroquad::prelude::*;

const TILE:f32=25.0;
const INK:Color=Color::new(0.16,0.12,0.08,1.0);
const PAPER:Color=Color::new(0.78,0.69,0.54,1.0);
const PANEL:Color=Color::new(0.86,0.77,0.62,0.97);
const GREEN:Color=Color::new(0.20,0.29,0.20,1.0);
const BLUE:Color=Color::new(0.29,0.39,0.47,1.0);
const RUST:Color=Color::new(0.57,0.25,0.14,1.0);

fn hash(x:i32,y:i32,s:u32)->u32{let mut n=(x as u32).wrapping_mul(374761393).wrapping_add((y as u32).wrapping_mul(668265263)).wrapping_add(s.wrapping_mul(1442695041));n=(n^(n>>13)).wrapping_mul(1274126177);n^(n>>16)}
fn river(x:i32,y:i32)->bool{let c=(x as f32*0.035).sin()*5.0+(x as f32*0.011).sin()*10.0;(y as f32-c).abs()<1.3}
fn road(x:i32,y:i32)->bool{(y-(x.div_euclid(22)*22+4)).abs()<1&&x.rem_euclid(22)<20||(x-(y.div_euclid(25)*25+6)).abs()<1&&y.rem_euclid(25)<22}
#[derive(Clone,Copy,PartialEq)] enum Kind{Water,Tree,Rock,Grass,Path,House,Ruin,Fire,Crop,Bridge}
fn kind(x:i32,y:i32)->Kind{
 if river(x,y){if y.rem_euclid(19)==0&&x.rem_euclid(13)<5{return Kind::Bridge}return Kind::Water}
 if road(x,y){return Kind::Path}
 let h=hash(x,y,31)%100;let grove=hash(x.div_euclid(5),y.div_euclid(5),8)%100;
 if grove>40&&h<73{Kind::Tree}else if h<4{Kind::Rock}else if h==5{Kind::Fire}else if h==6{Kind::House}else if h==7{Kind::Ruin}else if h<14{Kind::Crop}else{Kind::Grass}
}
fn glyph(k:Kind,x:i32,y:i32)->(&'static str,Color){match k{
 Kind::Water=>(if hash(x,y,2)%3==0{"~"}else{"="},BLUE),
 Kind::Tree=>(match hash(x,y,3)%4{0=>"Y",1=>"T",2=>"♣",_=>"♠"},GREEN),
 Kind::Rock=>(if hash(x,y,4)%2==0{"o"}else{"O"},INK),
 Kind::Path=>(if hash(x,y,5)%3==0{"·"}else{"."},Color::new(0.43,0.32,0.20,1.0)),
 Kind::House=>("H",RUST),Kind::Ruin=>("A",INK),Kind::Fire=>("*",RUST),Kind::Crop=>("v",GREEN),
 Kind::Bridge=>("#",INK),Kind::Grass=>(match hash(x,y,6)%5{0=>",",1=>"'",2=>".",3=>";",_=>" "},GREEN)
}}
fn walkable(k:Kind)->bool{!matches!(k,Kind::Water|Kind::Tree|Kind::Rock)}
struct Game{px:f32,py:f32,hp:i32,bag:i32,steps:u64,clock:f32,panel:i32,facing:(i32,i32),msg:String,msg_t:f32}
impl Game{
 fn new()->Self{Self{px:0.,py:0.,hp:8,bag:0,steps:0,clock:8.,panel:0,facing:(0,1),msg:"A quiet morning in the Paper Wilds.".into(),msg_t:4.}}
 fn movement(&mut self,dx:f32,dy:f32,dt:f32){if dx==0.&&dy==0.{return}let l=(dx*dx+dy*dy).sqrt();let sx=dx/l;let sy=dy/l;self.facing=(sx.signum() as i32,sy.signum() as i32);let nx=self.px+sx*3.0*dt;let ny=self.py+sy*3.0*dt;if walkable(kind(nx.floor() as i32,self.py.floor() as i32)){self.px=nx}if walkable(kind(self.px.floor() as i32,ny.floor() as i32)){self.py=ny}self.steps+=1;}
 fn interact(&mut self){let x=self.px.floor() as i32+self.facing.0;let y=self.py.floor() as i32+self.facing.1;self.msg=match kind(x,y){Kind::Rock=>{self.bag+=1;"You found a smooth stone.".into()},Kind::Crop=>{self.bag+=1;"You gathered wild grain.".into()},Kind::Fire=>{self.hp=8;"You rest. Health restored.".into()},Kind::House=>"An old field house.".into(),Kind::Ruin=>"KEEP WALKING.".into(),Kind::Water=>"The river is too deep.".into(),Kind::Tree=>"The forest is thick.".into(),_=>"Only wind and paper.".into()};self.msg_t=3.;}
}
fn panel(x:f32,y:f32,w:f32,h:f32){draw_rectangle(x,y,w,h,PANEL);draw_rectangle_lines(x,y,w,h,1.2,INK)}
fn txt(s:&str,x:f32,y:f32,size:f32){draw_text(s,x,y,size,INK)}
fn paper(){clear_background(PAPER);let w=screen_width();let h=screen_height();for i in 0..(h/5.) as i32{draw_line(0.,i as f32*5.,w,i as f32*5.,0.45,Color::new(0.35,0.24,0.14,0.10))}for i in 0..(w*h/6500.) as i32{let x=hash(i,17,3) as f32/u32::MAX as f32*w;let y=hash(i,29,4) as f32/u32::MAX as f32*h;draw_circle(x,y,0.6,Color::new(0.22,0.16,0.1,0.13))}}
fn cam(g:&Game)->Camera2D{let s=(screen_height()/720.).clamp(0.72,1.25);Camera2D{target:vec2(g.px*TILE,g.py*TILE),zoom:vec2(2./screen_width(),-2./screen_height())*TILE*s,..Default::default()}}
fn button(s:&str,x:f32,y:f32,w:f32,h:f32)->bool{let(m,n)=mouse_position();let hit=m>=x&&m<=x+w&&n>=y&&n<=y+h;draw_rectangle(x,y,w,h,if hit{Color::new(0.72,0.62,0.46,1.)}else{PANEL});draw_rectangle_lines(x,y,w,h,1.,INK);let fs=if h<34.{14.}else{18.};let tw=measure_text(s,None,fs as u16,1.).width;txt(s,x+(w-tw)/2.,y+h*0.68,fs);hit&&is_mouse_button_pressed(MouseButton::Left)}
fn held(x:f32,y:f32,w:f32,h:f32)->bool{let(a,b)=mouse_position();is_mouse_button_down(MouseButton::Left)&&a>=x&&a<=x+w&&b>=y&&b<=y+h}
fn hud(g:&Game){let w=screen_width();let h=screen_height();let m=if w<600.{9.}else{18.};let small=w<600.;let fs=if small{12.}else{18.};let pw=if small{195.}else{270.};let ph=if small{76.}else{102.};panel(m,m,pw,ph);txt(&format!("DAY 03 | {:02}:{:02}",g.clock as i32,(g.clock.fract()*60.) as i32),m+10.,m+fs+5.,fs);txt(&format!("HP [{}{}]","#".repeat(g.hp as usize),".".repeat((8-g.hp).max(0) as usize)),m+10.,m+fs*2.3,fs);txt(&format!("BAG: {}   X:{} Y:{}",g.bag,g.px.floor() as i32,g.py.floor() as i32),m+10.,m+fs*3.55,fs);let cx=w-m-38.;let cy=m+28.;txt("N",cx,cy-10.,fs);txt("W + E",cx-15.,cy+9.,fs*0.7);txt("S",cx,cy+25.,fs);
 let bh=if small{31.}else{43.};let by=h-m-bh;panel(m,by,w-m*2.,bh);txt(if small{"[E] INTERACT  [I] BAG  [M] MAP  [Q] LOG"}else{"[E] INTERACT   [I] INVENTORY   [M] MAP   [Q] JOURNAL"},m+9.,by+bh*0.68,if small{10.}else{15.});
 if g.msg_t>0.{let tw=measure_text(&g.msg,None,14,1.).width;let ww=(tw+20.).min(w-30.);let x=(w-ww)/2.;panel(x,ph+m*2.,ww,30.);txt(&g.msg,x+10.,ph+m*2.+20.,14.);}
}
#[macroquad::main("PAPERLAND")]
async fn main(){let mut g=Game::new();loop{
 let dt=get_frame_time().min(0.05);g.clock+=dt/120.;let mut dx=0.;let mut dy=0.;
 if is_key_down(KeyCode::A)||is_key_down(KeyCode::Left){dx-=1.}if is_key_down(KeyCode::D)||is_key_down(KeyCode::Right){dx+=1.}if is_key_down(KeyCode::W)||is_key_down(KeyCode::Up){dy-=1.}if is_key_down(KeyCode::S)||is_key_down(KeyCode::Down){dy+=1.}
 let bx=screen_width()-126.;let by=screen_height()-155.;if held(bx+40.,by,42.,40.){dy-=1.}if held(bx,by+42.,40.,40.){dx-=1.}if held(bx+82.,by+42.,40.,40.){dx+=1.}if held(bx+40.,by+84.,42.,40.){dy+=1.}
 g.movement(dx,dy,dt);if is_key_pressed(KeyCode::E){g.interact()}if is_key_pressed(KeyCode::I){g.panel=if g.panel==1{0}else{1}}if is_key_pressed(KeyCode::M){g.panel=if g.panel==2{0}else{2}}if is_key_pressed(KeyCode::Q){g.panel=if g.panel==3{0}else{3}}if g.msg_t>0.{g.msg_t-=dt}
 paper();set_camera(&cam(&g));let cx=g.px.floor() as i32;let cy=g.py.floor() as i32;let cols=(screen_width()/TILE/2.) as i32+3;let rows=(screen_height()/TILE/2.) as i32+3;
 for y in cy-rows..=cy+rows{for x in cx-cols..=cx+cols{let k=kind(x,y);let(ch,c)=glyph(k,x,y);let px=x as f32*TILE;let py=y as f32*TILE;draw_text(ch,px+5.,py+19.,if k==Kind::Tree{21.}else{17.},c);if k==Kind::House{draw_text("/\\",px-3.,py-1.,13.,INK);draw_text("|_|",px+1.,py+29.,13.,INK)}if k==Kind::Ruin{draw_text("|||",px-3.,py+29.,11.,INK)}if k==Kind::Bridge{draw_line(px,py+8.,px+TILE,py+8.,1.,INK);draw_line(px,py+17.,px+TILE,py+17.,1.,INK)}}}
 draw_text("@",g.px*TILE+3.,g.py*TILE+20.,23.,RUST);set_default_camera();hud(&g);
 if button("^",bx+40.,by,42.,40.){}if button("<",bx,by+42.,40.,40.){}if button(">",bx+82.,by+42.,40.,40.){}if button("v",bx+40.,by+84.,42.,40.){}if button("E",screen_width()-72.,by-38.,40.,32.){g.interact()}
 let ty=screen_height()-75.;if button("BAG",10.,ty,48.,25.){g.panel=if g.panel==1{0}else{1}}if button("MAP",63.,ty,48.,25.){g.panel=if g.panel==2{0}else{2}}if button("LOG",116.,ty,48.,25.){g.panel=if g.panel==3{0}else{3}}
 if g.panel!=0{let w=screen_width();let h=screen_height();draw_rectangle(0.,0.,w,h,Color::new(0.12,0.09,0.06,0.4));let pw=(w-36.).min(420.);let ph=220.;let x=(w-pw)/2.;let y=(h-ph)/2.;panel(x,y,pw,ph);txt(match g.panel{1=>"BACKPACK",2=>"FIELD MAP",_=>"TRAVEL JOURNAL"},x+18.,y+32.,22.);match g.panel{1=>{txt(&format!("Collected supplies: {}",g.bag),x+18.,y+75.,17.);txt("Stone, grain and little discoveries.",x+18.,y+103.,15.)},2=>{txt("You are @ in the Paper Wilds.",x+18.,y+75.,16.);txt("Rivers, forests and old paths surround you.",x+18.,y+103.,15.)},_=>{txt("Every path leaves a mark on the page.",x+18.,y+75.,16.);txt(&format!("Steps travelled: {}",g.steps),x+18.,y+103.,16.)}}if button("CLOSE",w/2.-45.,h/2.+72.,90.,30.){g.panel=0}}
 next_frame().await;
}}