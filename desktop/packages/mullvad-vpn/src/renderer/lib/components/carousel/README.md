# Carousel

Uses the `Gallery` atom.

## Example

```tsx
<Carousel aria-label="carousel-aria-label">
  <Carousel.Slides>
    <Carousel.Slides.Slide>
      <Carousel.Slides.Slide.Image source="carousel-image-1" alt="Description of image 1" />
      <Carousel.Slides.Slide.TextGroup>
        <Carousel.Slides.Slide.Text>Slide 1</Carousel.Slides.Slide.Text>
        <Carousel.Slides.Slide.Text>
          Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        </Carousel.Slides.Slide.Text>
      </Carousel.Slides.Slide.TextGroup>
    </Carousel.Slides.Slide>
    <Carousel.Slides.Slide>
      <Carousel.Slides.Slide.Image source="carousel-image-2" alt="Description of image 2" />
      <Carousel.Slides.Slide.TextGroup>
        <Carousel.Slides.Slide.Text>Slide 2</Carousel.Slides.Slide.Text>
        <Carousel.Slides.Slide.Text>
          Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
        </Carousel.Slides.Slide.Text>
      </Carousel.Slides.Slide.TextGroup>
    </Carousel.Slides.Slide>
  </Carousel.Slides>
  <Carousel.Controls>
    <Carousel.Controls.Indicators />
    <Carousel.Controls.ControlGroup>
      <Carousel.Controls.PrevButton />
      <Carousel.Controls.NextButton />
    </Carousel.Controls.ControlGroup>
  </Carousel.Controls>
</Carousel>
```
