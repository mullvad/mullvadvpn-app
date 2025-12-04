# Carousel

Uses the `Gallery` atom.

## Example

```tsx
<Carousel aria-label="carousel-aria-label">
  <Carousel.Slides>
    <Carousel.Slide>
      <Carousel.Image source="carousel-image-1" alt="Description of image 1" />
      <Carousel.TextGroup>
        <Carousel.Text>Slide 1</Carousel.Text>
        <Carousel.Text>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</Carousel.Text>
      </Carousel.TextGroup>
    </Carousel.Slide>
    <Carousel.Slide>
      <Carousel.Image source="carousel-image-2" alt="Description of image 2" />
      <Carousel.TextGroup>
        <Carousel.Text>Slide 2</Carousel.Text>
        <Carousel.Text>
          Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
        </Carousel.Text>
      </Carousel.TextGroup>
    </Carousel.Slide>
  </Carousel.Slides>
  <Carousel.Controls>
    <Carousel.Indicators />
    <Carousel.ControlGroup>
      <Carousel.PrevButton />
      <Carousel.NextButton />
    </Carousel.ControlGroup>
  </Carousel.Controls>
</Carousel>
```
