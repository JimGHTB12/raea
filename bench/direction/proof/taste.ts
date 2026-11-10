// The proof's taste (passed with --taste): one line raea measures, run as the named check of its form (text-above)
// beside the project's own checks.
import { taste } from "raea/brief";

export default taste({
  title: "Proof taste",
  sections: [{ heading: "Text", rules: [{ text: "Captions sit above three quarters of the frame.", check: { form: "text-above", value: 75 } }] }],
});
