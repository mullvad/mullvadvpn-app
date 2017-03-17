import { expect } from 'chai';
import TrayAnimation from '../app/lib/tray-animation';
import { nativeImage } from 'electron';

describe('lib/tray-animation', function() {
  this.timeout(1000);
  
  let animation;

  beforeEach(() => {
    const images = [1, 2, 3, 4, 5].map(() => nativeImage.createEmpty());
    animation = new TrayAnimation(images);
    animation.speed = 1;
  });

  afterEach(() => {
    animation.stop();
  });
  
  it('should play sequence', (done) => {
    let seq = [];

    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };

    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
      expect(animation._currentFrame).to.be.equal(4);
      done();
    };

    animation.play();
  });

  it('should play sequence in reverse', (done) => {
    let seq = [];

    animation.onFrame = () => {
      seq.push(animation._currentFrame);
    };

    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2, 1, 0]);
      expect(animation._currentFrame).to.be.equal(0);
      done();
    };

    animation.reverse = true;
    animation.play();
  });

  it('should play sequence on repeat', (done) => {
    const expectedFrames = [0, 1, 2, 3, 4, 0, 1, 2, 3, 4];
    let receivedFrames = [];

    animation.onFrame = () => {
      if(receivedFrames.length === expectedFrames.length) {
        expect(receivedFrames).to.be.deep.equal(expectedFrames);
        done();
      } else {
        receivedFrames.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.play();
  });

  it('should play sequence on repeat in reverse', (done) => {
    const expectedFrames = [4, 3, 2, 1, 0, 4, 3, 2, 1, 0];
    let receivedFrames = [];

    animation.onFrame = () => {
      if(receivedFrames.length === expectedFrames.length) {
        expect(receivedFrames).to.be.deep.equal(expectedFrames);
        done();
      } else {
        receivedFrames.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.reverse = true;
    animation.play();
  });

  it('should alternate sequence', (done) => {
    const expectedFrames = [0, 1, 2, 3, 4, 3, 2, 1, 0];
    let receivedFrames = [];

    animation.onFrame = () => {
      if(receivedFrames.length === expectedFrames.length) {
        expect(receivedFrames).to.be.deep.equal(expectedFrames);
        done();
      } else {
        receivedFrames.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.alternate = true;
    animation.play();
  });

  it('should alternate reverse sequence', (done) => {
    const expectedFrames = [4, 3, 2, 1, 0, 1, 2, 3, 4];
    let receivedFrames = [];

    animation.onFrame = () => {
      if(receivedFrames.length === expectedFrames.length) {
        expect(receivedFrames).to.be.deep.equal(expectedFrames);
        done();
      } else {
        receivedFrames.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.reverse = true;
    animation.alternate = true;
    animation.play();
  });

});