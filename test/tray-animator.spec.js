import { expect } from 'chai';
import { TrayAnimator,  } from '../app/lib/tray-animator';
import { TrayAnimation } from '../app/lib/tray-animation';
import { nativeImage } from 'electron';

describe('lib/tray-animator', function() {
  this.timeout(1000);
  
  let animation, animator;

  beforeEach(() => {
    const images = [1, 2, 3, 4, 5].map(() => nativeImage.createEmpty());
    animation = new TrayAnimation(images);
    animation.speed = 1;
  });

  afterEach(() => {
    if(animator.isStarted) {
      animator.stop();
    }
  });

  it('should play sequence', (done) => {
    let seq = [];

    const tray = {
      setImage: () => {
        if(animation.isFinished) {
          expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
          expect(animation._currentFrame).to.be.equal(4);
          done();
        } else {
          seq.push(animation._currentFrame);
        }
      }
    };

    animator = new TrayAnimator(tray, animation);
    animator.start();
  });

  it('should play sequence in reverse', (done) => {
    let seq = [];

    const tray = {
      setImage: () => {
        if(animation.isFinished) {
          expect(seq).to.be.deep.equal([4, 3, 2, 1, 0]);
          expect(animation._currentFrame).to.be.equal(0);
          done();
        } else {
          seq.push(animation._currentFrame);
        }
      }
    };

    animation.reverse = true;
    animator = new TrayAnimator(tray, animation);
    animator.start();
  });

  it('should play sequence on repeat', (done) => {
    const expectedFrames = [0, 1, 2, 3, 4, 0, 1, 2, 3, 4];
    let receivedFrames = [];

    const tray = {
      setImage: () => {
        if(receivedFrames.length === expectedFrames.length) {
          expect(receivedFrames).to.be.deep.equal(expectedFrames);
          animator.stop();
          done();
        } else {
          receivedFrames.push(animation._currentFrame);
        }
      }
    };

    animation.repeat = true;
    animator = new TrayAnimator(tray, animation);
    animator.start();
  });

  it('should play sequence on repeat in reverse', (done) => {
    const expectedFrames = [4, 3, 2, 1, 0, 4, 3, 2, 1, 0];
    let receivedFrames = [];

    const tray = {
      setImage: () => {
        if(receivedFrames.length === expectedFrames.length) {
          expect(receivedFrames).to.be.deep.equal(expectedFrames);
          animator.stop();
          done();
        } else {
          receivedFrames.push(animation._currentFrame);
        }
      }
    };

    animation.repeat = true;
    animation.reverse = true;
    animator = new TrayAnimator(tray, animation);
    animator.start();
  });

  it('should alternate sequence', (done) => {
    const expectedFrames = [0, 1, 2, 3, 4, 3, 2, 1, 0];
    let receivedFrames = [];

    const tray = {
      setImage: () => {
        if(receivedFrames.length === expectedFrames.length) {
          expect(receivedFrames).to.be.deep.equal(expectedFrames);
          animator.stop();
          done();
        } else {
          receivedFrames.push(animation._currentFrame);
        }
      }
    };

    animation.repeat = true;
    animation.alternate = true;
    animator = new TrayAnimator(tray, animation);
    animator.start();
  });

  it('should alternate reverse sequence', (done) => {
    const expectedFrames = [4, 3, 2, 1, 0, 1, 2, 3, 4];
    let receivedFrames = [];

    const tray = {
      setImage: () => {
        if(receivedFrames.length === expectedFrames.length) {
          expect(receivedFrames).to.be.deep.equal(expectedFrames);
          animator.stop();
          done();
        } else {
          receivedFrames.push(animation._currentFrame);
        }
      }
    };

    animation.repeat = true;
    animation.reverse = true;
    animation.alternate = true;
    animator = new TrayAnimator(tray, animation);
    animator.start();
  });

});