# Ray Tracing

This is my implementation of a basic ray tracing program, following along with the [Ray Tracing in One Weekend](https://raytracing.github.io/) series of books.

I implemented the code in the book in Rust instead of C++.
The commit history roughly corresponds with completing the sections of the books, with each commit usually generating a new image.
There are also `git` tags for each section for which a new image is generated at the end, which are prefixed by an abbreviation for the book.
As the program grows in complexity at each step, things are refactored appropriately.

Each image can be viewed by redirecting the program output to a `.ppm` file and opening that file in your favorite image viewer.
Run the program with `-h` to see other options.
