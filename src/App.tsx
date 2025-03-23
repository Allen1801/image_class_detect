import React, { useState } from 'react'
import axios from "axios";
import { loadingimg } from './assets/images'
import './App.css'

function App() {
  const [imgSrc, setImgSrc] = React.useState<string>("");
  const [processedImgSrc, setProcessedImgSrc] = useState<string | null>(null);

  const handleImageUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      const file_reader = new FileReader();
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

  const sendImageToBackend = async (file: File) => {
    const formData = new FormData();
    formData.append("image", file);

    try {
      const response = await axios.post("http://127.0.0.1:8000/detect_image", formData, {
        headers: {
          "Content-Type": "multipart/form-data",
        },
      });

      console.log("OCR Result:", response.data);
      setImgSrc(`data:image/png;base64,${response.data}`); // Display processed image
    } catch (error) {
      console.error("Error sending image:", error);
    }
  };



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
          <p>Image Size: </p>
          <p>object Detected: </p>
        </div>

          <button className='submit-btn'>Submit</button>

      </div>
    </div>
    </>
  )
}

export default App
