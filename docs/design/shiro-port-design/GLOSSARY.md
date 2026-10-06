# Speech alignment

SHIRO aligns a known phoneme sequence with a speech recording and trains the statistical models used for that alignment.

## Language

**Forced alignment**:
The assignment of time boundaries to a known phoneme sequence in a speech recording.
_Avoid_: Speech recognition, transcription

**Phoneme**:
A speech sound category represented in the transcription supplied for alignment.

**Observation sequence**:
An ordered sequence of acoustic feature vectors describing a recording at successive frames.
_Avoid_: Raw waveform

**Hidden semi-Markov model**:
A statistical model of hidden states with explicit state-duration distributions and observation-emission distributions.
_Avoid_: Hidden Markov model when explicit duration distributions are meant

**Segmentation**:
A state sequence associated with a recording, with time boundaries when available.
_Avoid_: Transcription when state assignments and timing are meant

**Phone map**:
The mapping from phonemes to their model states and transition topology.

**Re-estimation**:
The update of model parameters from observations and state-sequence constraints.

**Flat-start initialization**:
Model initialization performed without a previously estimated alignment.
