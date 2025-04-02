import React, { useState } from 'react'
import axios from "axios";
import { loadingimg } from './assets/images'
import './App.css'

function App() {
  const [imgSrc, setImgSrc] = useState<string>("");
  const [imgLabel, setImgLabel] = useState<string>("");
  const [imgSize, setImgSize] = useState<string>("");

  const handleImageUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const file_reader = new FileReader();
      const fileSizeInKB = (file.size / 1024).toFixed(2); // Size in KB
      const fileSizeInMB = (file.size / (1024 * 1024)).toFixed(2); // Size in MB
      setImgSize(`${fileSizeInKB} KB (${fileSizeInMB} MB)`); // Set size in both KB and MB
      file_reader.onload = () => {
        setImgSrc(file_reader.result as string);
        sendImageToBackend(file);
      };
      file_reader.onerror = (error) => {
        console.log(error);
      }
      file_reader.readAsDataURL(file);


    }
  }

  async function sendImageToBackend(file: File) {
    const fileReader = new FileReader();
    
    fileReader.onload = async () => {
        const base64Image = fileReader.result as string;
        try {
            const payload = { image: base64Image };  // Wrap in an object
            console.log("Sending data:", payload);  // Debugging

            const response = await axios.post("https://image-class-detect.onrender.com/detect_image", 
                payload,
                {
                    headers: { "Content-Type": "application/json" }
                }
            );
            console.log("Response image:", response.data);
            setImgSrc(`data:image/png;base64,${response.data.image}`);  // Assuming the response contains the image
            setImgLabel(
              Array.isArray(response.data.labels)
                  ? [...new Set(response.data.labels)].join(", ") // Remove duplicates and join
                  : response.data.labels
          );
        } catch (error) {
            console.error("Error:", error);
        }
    };

    fileReader.onerror = (error) => {
        console.log("File read error:", error);
    };

    fileReader.readAsDataURL(file);  // Convert to base64
}

  return (
    <>
    <h2>Image Classification & Detection</h2>
    <div className="card">
      <div className="container">
        <div className='input-file'>
          <input 
            type="file" 
            name="file" 
            id="file" 
            accept="image/*"
            onChange={handleImageUpload}/>
        </div>
        <div className='image-box'>
          <img src={imgSrc || loadingimg} alt="" className='image'/>
        </div>
        <div className='description'>
          <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. In erat nulla, 
            egestas at ipsum vitae, dictum pretium nibh. Fusce hendrerit.</p>
        </div>
        <div className='result'>
          <h2>Image Result</h2>
          <p>Image Size: {imgSize}</p>
          <p>Object Detected: {imgLabel}</p>
        </div>

      </div>
    </div>
    </>
  )
}

export default App
