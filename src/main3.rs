mod screen;

fn main(){
    let args: Vec<String> = env::args().collect();
    let state = screen_state{};
    std::thread::spawn(move ||{

        loop_logic(args,state);

    });

    
    
}