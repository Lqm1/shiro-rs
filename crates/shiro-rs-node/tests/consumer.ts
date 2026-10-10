import { Model, TrainingOptions, Datasets, version } from '..';
function train(model: Model, data: Datasets): Uint8Array {
  return model.train_with_progress(data, new TrainingOptions(), report => {
    const iteration: number = report.iteration;
    void iteration;
  }).model().write();
}
const packageVersion: string = version();
void [train, packageVersion];
