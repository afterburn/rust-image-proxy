# rust-image-proxy

A simple image proxy built with [Actix Web](https://actix.rs/).

### Features
- Convert images to .webp format.
- Resize images.

### How it works
On first hit images are downloaded via [reqwest](https://github.com/seanmonstar/reqwest) and stored on disk. Subsequent hits will be served directly from disk.<br>
Image conversion is achieved through [libwebp](https://github.com/webmproject/libwebp) via the Rust implementation [webp](https://github.com/jaredforth/webp).<br>
Resizing works via [image](https://github.com/image-rs/image).

### Configuration
```
// .env
PORT=<your-desired-port>
```

### Example usage
```
GET /?url=https%3A%2F%2Fplacekitten.com%2F800%2F600&w=300
```

| Param | Description                     |
|-------|---------------------------------|
| url   | encoded url of the target image |
| w     | desired width for the result    |
| h     | desired height for the result   |

Omitting either `w` or `h` will preserve original aspect ratio of the image.

Result:

![image](https://user-images.githubusercontent.com/10757768/200688034-4ad18c6c-a905-4167-a393-69908fd532d0.png)
