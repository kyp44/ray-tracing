# Ray Tracing

This is my implementation of a basic ray tracing program, following along with the [Ray Tracing in One Weekend](https://raytracing.github.io/) series of books.

I implemented the code in the book in Rust instead of C++.
The commit history roughly corresponds with completing the sections of the book, with each commit usually generating a new image.
As the program grows in complexity at each step, things are refactored appropriately.

Each image can be viewed by redirecting the program output to a `.ppm` file and opening that file in your favorite image viewer.
Run the program with `-h` to see other options.
