use cocoa::base::*;
use cocoa::foundation::*;

pub fn load(path: &str) -> Result<Vec<id>, &'static str> {
    unsafe {
        let filename = NSString::alloc(nil).init_str(path);
        let nsdata: id = msg_send![class("NSData"), dataWithContentsOfFile: filename];
        let nsnib: id = msg_send![class("NSNib"), alloc];
        msg_send![nsnib, initWithNibData:nsdata bundle:nil];

        let objects: id = msg_send![class("NSArray"), alloc];
        msg_send![objects, init];

        let success: BOOL = msg_send![nsnib, instantiateWithOwner:nil topLevelObjects:&objects];
        if success == NO {
            return Err("Can't load nib file");
        }

        let count: NSInteger = msg_send![objects, count];

        let mut instances = Vec::new();

        for i in 0..count {
            let instance: id = msg_send![objects, objectAtIndex:i];
            instances.push(instance);
        }

        Ok(instances)
    }
}
