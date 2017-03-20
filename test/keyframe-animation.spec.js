import { expect } from 'chai';
import KeyframeAnimation from '../app/lib/keyframe-animation';
import { nativeImage } from 'electron';

describe('lib/keyframe-animation', function() {
  this.timeout(1000);
  
  let animation, seq;

  beforeEach(() => {
    const images = [1, 2, 3, 4, 5].map(() => nativeImage.createEmpty());
    animation = new KeyframeAnimation(images);
    animation.speed = 1;

    seq = [];
  });

  afterEach(() => {
    animation.stop();
  });
  
  it('should play sequence', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
      expect(animation._currentFrame).to.be.equal(4);
      done();
    };

    animation.play();
  });

  it('should play one frame', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([3]);
      expect(animation._currentFrame).to.be.equal(3);
      done();
    };

    animation.play({ startFrame: 3, endFrame: 3 });
  });

  it('should play sequence with custom frames', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([2, 3, 4]);
      expect(animation._currentFrame).to.be.equal(4);
      done();
    };

    animation.play({
      startFrame: 2,
      endFrame: 4
    });
  });

  it('should play sequence with custom frames in reverse', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2]);
      expect(animation._currentFrame).to.be.equal(2);
      done();
    };

    animation.reverse = true;
    animation.play({
      startFrame: 4,
      endFrame: 2
    });
  });

  it('should begin from current state starting below range', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3, 4]);
      expect(animation._currentFrame).to.be.equal(4);
      done();
    };

    animation._currentFrame = 0;
    animation._isFirstRun = false;

    animation.play({
      beginFromCurrentState: true,
      startFrame: 3,
      endFrame: 4
    });
  });

  it('should begin from current state starting below range reverse', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([0, 1, 2, 3]);
      expect(animation._currentFrame).to.be.equal(3);
      done();
    };

    animation._currentFrame = 0;
    animation._isFirstRun = false;
    animation.reverse = true;

    animation.play({
      beginFromCurrentState: true,
      startFrame: 3,
      endFrame: 4
    });
  });

  it('should begin from current state starting above range', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2]);
      expect(animation._currentFrame).to.be.equal(2);
      done();
    };

    animation._currentFrame = 4;
    animation._isFirstRun = false;

    animation.play({
      beginFromCurrentState: true,
      startFrame: 1,
      endFrame: 2
    });
  });

  it('should begin from current state starting above range reverse', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
    animation.onFinish = () => {
      expect(seq).to.be.deep.equal([4, 3, 2, 1]);
      expect(animation._currentFrame).to.be.equal(1);
      done();
    };

    animation._currentFrame = 4;
    animation._isFirstRun = false;
    animation.reverse = true;

    animation.play({
      beginFromCurrentState: true,
      startFrame: 1,
      endFrame: 3
    });
  });

  it('should play sequence in reverse', (done) => {
    animation.onFrame = () => seq.push(animation._currentFrame);
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

    animation.onFrame = () => {
      if(seq.length === expectedFrames.length) {
        expect(seq).to.be.deep.equal(expectedFrames);
        done();
      } else {
        seq.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.play();
  });

  it('should play sequence on repeat in reverse', (done) => {
    const expectedFrames = [4, 3, 2, 1, 0, 4, 3, 2, 1, 0];

    animation.onFrame = () => {
      if(seq.length === expectedFrames.length) {
        expect(seq).to.be.deep.equal(expectedFrames);
        done();
      } else {
        seq.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.reverse = true;
    animation.play();
  });

  it('should alternate sequence', (done) => {
    const expectedFrames = [0, 1, 2, 3, 4, 3, 2, 1, 0];

    animation.onFrame = () => {
      if(seq.length === expectedFrames.length) {
        expect(seq).to.be.deep.equal(expectedFrames);
        done();
      } else {
        seq.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.alternate = true;
    animation.play();
  });

  it('should alternate reverse sequence', (done) => {
    const expectedFrames = [4, 3, 2, 1, 0, 1, 2, 3, 4];

    animation.onFrame = () => {
      if(seq.length === expectedFrames.length) {
        expect(seq).to.be.deep.equal(expectedFrames);
        done();
      } else {
        seq.push(animation._currentFrame);
      }
    };

    animation.repeat = true;
    animation.reverse = true;
    animation.alternate = true;
    animation.play();
  });

});