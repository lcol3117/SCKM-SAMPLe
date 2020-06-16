// Import required external packages
// Rayon is for parallel mapping
extern crate rayon;

// Use required packages and standard library traits
// For concurrency
use std::thread;
use std::thread::*;
// For parallel mapping
use rayon::prelude;
use rayon::prelude::*;

// Trait for SCKM Model itself
pub trait SCKMModel {
  // Trains the SCKM model
  // Training must be a method that takes the eta hyperparameter
  // Training must return the Some(Trained) or None if it fails
  // Its work is stored in the self.result vector of cluster IDs
  fn train(&mut self, eta: u32) -> option<Trained>,
  // Checks if a and b are in the same cluster
  // Takes a and b (points in a boolean space) as Vectors of booleans
  // Returns Some(ConnectEnum) if self.trained is DoneEnum::done, otherwise None
  // Its work is stored in the return, and must not mutate self
  // Note that a and b are Vec<bool> not LabelBoolPoint
  // This is because we do not use label information
  fn same_cluster(&self, a: Vec<bool>, b: Vec<bool>) -> option<ConnectEnum>,
  // This is used to update the SCKM.data value
  // Do not do this directly
  // This will automatically await the SCKM.trained TaskState
  // It will also set the SCKM.trained TaskState
  // Note that it calls SCKM::new, and then passes on the fields
  fn update_data(&mut self, new_data: Vec<LabelBoolPoint>)
}

// Trait for the constructor of an SCKM Model
pub trait SCKMModelConstructor {
  // The constructor takes the data and returns a Model
  // Note that the hyperparameter eta is given at training
  fn new(given_data: Vec<LabelBoolPoint>) -> SCKM
}

pub struct SCKM {
  // Vector of points in a boolean space, use LabelBoolPoint struct
  data: Vec<LabelBoolPoint>,
  // Vector of Some(cluster center) or None if not yet found
  result: Vec<option<BoolPoint>>,
  // Number of cluster centers, None if not yet found, and the checking job
  num_centers: JobU8,
  // Is the training done, ready, or pending
  trained: TaskState
}

// Constructor impl block, see 
impl SCKMModelConstructor for SCKM {
  // The constructor
  fn new(given_data: Vec<LabelBoolPoint>) -> self {
    // Build the SCKM object
    SCKM {
      data: given_data // Use given data
      result: intial_result, // Use generated cluster IDs
      trained: TaskState::ready // Ready to train
    }
  }
}

// Trait functions, see SCKMModel
impl SCKMModel for SCKM {
  // The train function, see SCKMModel
  fn train(&mut self, eta: u32) -> option<Trained> {
    // Check that self.trained is TaskState::ready
    if self.trained != TaskState::ready {
      return None // Return None
    }
    // Set that the train task is pending
    self.trained = TaskState::pending;
    // Reset the JobU8 num_centers to 0 and mark as pending
    self.num_centers = JobU8 {
      num: 0_u8,
      job: TaskState::pending
    }
    // Iterate until deemed complete by SCKM::training_iteration
    while self.trained == TaskState::pending {
      // Need to pass on eta, it is not a property
      self.training_iteration(eta: u32); // Call SCKM::training_iteration
    };
    // Return Some of the Trained unit struct, to represent completion
    // Note that the SCKM::trained property is set to TaskState::done
    // This is done by the SCKM::training_iteration method
    return Some(Trained)
  }
  
  // The same_cluster function, see SCKMModel
  fn same_cluster(&self, a: Vec<bool>, b: Vec<bool>) -> option<ConnectEnum> {
    // Unwrap the Vec<option<BoolPoint>> result, which represents centers
    let raw_cluster_centers = self.result // The wrapped cluster centers
      .iter() // Not parallel, that causes issues with panic::catch_unwind
      .map(|x| {(*x) // Deference the BoolPoint struct
        .clone() // Clone it to avoid pointer collsion
        .unwrap() // Convert E: option<BoolPoint> to E: BoolPoint
      })
      .collect::<Vec<BoolPoint>>(); // Collect the par_iter into a Vec
    // Use CenterBasedClustering to check for same cluster
    CenterBasedClustering::same_cluster(a, b, raw_cluster_centers)
  }
  
  // The update_data function, see SCKMModel
  fn update_data(&mut self, newdata: Vec<LabelBoolPoint>) {
    // Await the self.trained TaskState to be not pending
    while self.trained == TaskState::pending {};
    // Set the self.trained TaskState to be pending
    self.trained = TaskState::pending;
    // Generate a new SCKM object with the desired fields
    let new_SCKM_object = SCKM::new(newdata);
    // Update the data field
    self.data = new_SCKM_object.data;
    // Update the result field
    self.result = new_SCKM_object.result;
    // Set self.trained TaskState to ready
    self.trained = TaskState::ready;
    // Implicitly return unit
  }
}

// Internal training methods, see SCKM::train
impl SCKM {
  // Training iteration, called by SCKM::train, an impl of SCKMModel trait
  fn training_iteration(&mut self, eta: u32) {
    // TODO
  }
}

// Represent a potentially labeled point in boolean space
struct LabelBoolPoint {
  data: BoolPoint, // The point itself
  label: option<LabelEnum> // Some(LabelEnum) if labeles, otherwise None
}

// Represent a point in boolean space
struct BoolPoint {
  point: Vec<bool> // The point itself
}

// Represent the labels used in SAMPLe
enum LabelEnum {
  malware, // Malicious packages
  accept // Acceptable packages
}

// Implement copy and clone traits for LabelEnum
impl Copy for LabelEnum {}
impl Clone for LabelEnum {
  fn clone(&self) -> self {
    *self // Just return the enum value itself
  }
}

// Represent the state of a task
enum TaskState {
  done, // The task is complete
  ready, // The task has not been started
  pending // The task is currently running
}

// Implement copy and clone traits for TaskState
impl Copy for TaskState {}
impl Clone for TaskState {
  fn clone(&self) -> self {
    *self // Just return the enum value itself
  }
}

// Represent connectivity
enum ConnectEnum {
  linked, // The points are linked, in the same cluster
  seperate // The points are in seperate clusters
}

// Implement copy and clone traits for ConnectEnum
impl Copy for ConnectEnum {}
impl Clone for ConnectEnum {
  fn clone(&self) -> self {
    *self // Just return the enum value itself
  }
}

// Represent a u8 associated with a job
struct JobU8 {
  num: option<u8> // The u8 number value
  job: TaskState // The state of the associated task
}

// Implement Copy and Clone traits for JobU8
impl Copy for JobU8 {}
impl Clone for JobU8 {
  fn clone(&self) -> self {
    *self // Just return the enum value itself
  }
}

// new and from for JobU8
impl JobU8 {
  // The constructor
  fn new(new_num: option<u8>, new_job: TaskState) -> JobU8 {
    // Build the JobU8 object
    JobU8 {
      num: new_num, // Use given option<u8>
      job: new_job // Use given TaskState
    }
  }
  
  fn make() -> JobU8 {
    JobU8::new(None, TaskState::ready) // Initial state of a JobU8
  }
}

struct Trained; // Unit struct, used to represent training attempt is complete
