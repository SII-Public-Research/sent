/* USER CODE BEGIN Header */
/**
  ******************************************************************************
  * @file           : main.c
  * @brief          : Main program body
  ******************************************************************************
  * @attention
  *
  * <h2><center>&copy; Copyright (c) 2021 STMicroelectronics.
  * All rights reserved.</center></h2>
  *
  * This software component is licensed by ST under BSD 3-Clause license,
  * the "License"; You may not use this file except in compliance with the
  * License. You may obtain a copy of the License at:
  *                        opensource.org/licenses/BSD-3-Clause
  *
  ******************************************************************************
  */
/*
 * 	INTRO:
	this is the SENT driver developped by David KAMANA with the help of Ahmed TOUMI in the process
	the lines generate by stm32cubeIDE are not commented
	Some comments about data types:

	- In the DMA data processing, the uint32_t is used  because in the hal library for dma, the start IC allows only "uint32_t"

	*/
	/* USER CODE END Header */
/* Includes ------------------------------------------------------------------*/
#include "main.h"

/* Private includes ----------------------------------------------------------*/
/* USER CODE BEGIN Includes */
#include "stdio.h"
/* USER CODE END Includes */

/* Private typedef -----------------------------------------------------------*/
/* USER CODE BEGIN PTD */

/* USER CODE END PTD */

/* Private define ------------------------------------------------------------*/
/* USER CODE BEGIN PD */

/*for computing the number of ticks : */
#define TIMFREQ  64000000 // the timer's clock frequency
#define PRESCALAR  1 //the timer's prescaler
#define TICKS_OF_SYNCHRO 56 //number of ticks in synchro
#define SIZE_TAB_ticks 16 //Size of the table of ticks

/*for DMA usage:*/

#define TIM_DMA_Buffer_SIZE 19 //the DMA buffer's size
#define TIM_DMA_FRAME_ticks_SIZE 9 //the size of the table of ticks corresponding to the values  in the DMA buffer

/*for SENT data processing:*/
#define SENT_data_SIZE 8 //the number of SENT data in a frame
#define LOWEST_TICKS 12 //the lowest number of ticks in a SENT frame
/* USER CODE END PD */

/* Private macro -------------------------------------------------------------*/
/* USER CODE BEGIN PM */

/* USER CODE END PM */

/* Private variables ---------------------------------------------------------*/
TIM_HandleTypeDef htim2;
DMA_HandleTypeDef hdma_tim2_ch1;

UART_HandleTypeDef huart2;

/* USER CODE BEGIN PV */

/*For ticks and frame ckeck:*/
//the table containing  the number of ticks to compare to:
float  tab_ticks[] = {56.0,12.0,13.0,14.0,15.0,16.0,17.0,18.0,19.0,20.0,21.0,22.0,23.0,24.0,25.0,26.0,27.0};
uint16_t NB_frames_ok = 0;//the number of frames that starts with the synchro data, used for debug purposes
uint8_t flag_frame_ok = 0;//the flag that is set if a frame starts with synchro data


/*for DMA:*/
uint32_t TICKS_tab_DMA[TIM_DMA_Buffer_SIZE];//a table that stores the number of ticks per  DMA Data
uint32_t tab_val_DMA[TIM_DMA_Buffer_SIZE];//the table that acts as the destination for DMA data transfer
uint32_t tab_vals[TIM_DMA_FRAME_ticks_SIZE];//the table of values that contain the number of ticks per DMA data if their frame starts with synchro data
uint16_t counter_DMA_requests = 0;//number of times DMA requests are executed i.e after 10 data transfer
HAL_StatusTypeDef DMA_available;//the variable that helps to check whether the DMA is available for the next transfer

/*for SENT protocol:*/
uint32_t SENT_data_tab[SENT_data_SIZE];
/* USER CODE END PV */

/* Private function prototypes -----------------------------------------------*/
void SystemClock_Config(void);
static void MX_GPIO_Init(void);
static void MX_USART2_UART_Init(void);
static void MX_DMA_Init(void);
static void MX_TIM2_Init(void);

/* USER CODE BEGIN PFP */

/*For Computing number of ticks:*/
void tick_data_process(uint32_t* tab_src,  uint32_t* tab_dst);//assigns the DMA data to the corresponding number of ticks as long as the serie of  data starts with synchro
void fill_table_ticks(uint32_t* tab_source, uint32_t* tab_dest, uint32_t size_buffer);//function that computes the ticks corresponding to DMA data as long as the serie of  data starts with synchro
uint32_t compute_tick(float _width, float TAB_ticks[]);//function that computes the number of ticks
void BufferDMA_to_ticks(uint32_t* tab_1 , uint32_t* tab_2);//function that assigns the DMA data to the number of ticks
/*For Data processing :*/
void initialise_tab(uint32_t* tab, int len_tab);//function that reset the table values to zero
uint8_t set_flag_data_ok(uint32_t* tab_ticks);//function that sets a flag to indicate that the frame has as first data, the synchro data
float absolute_val(float x);//function that computes the absolute value of a number.
void data_SENT(uint32_t* tab_DMA_ticks, uint32_t* tab_data_SENT);//computes the SENT protocol data after we have got the number of ticks in a SENT frame
/*
Note that: I had to write the absolute value computing function because the "abs" function that already
exist in C  didnot give expected results.
*/

/* USER CODE END PFP */

/* Private user code ---------------------------------------------------------*/
/* USER CODE BEGIN 0 */

/* USER CODE END 0 */

/**
  * @brief  The application entry point.
  * @retval int
  */
int main(void)
{
  /* USER CODE BEGIN 1 */

  /* USER CODE END 1 */

  /* MCU Configuration--------------------------------------------------------*/

  /* Reset of all peripherals, Initializes the Flash interface and the Systick. */
  HAL_Init();

  /* USER CODE BEGIN Init */

  /* USER CODE END Init */

  /* Configure the system clock */
  SystemClock_Config();

  /* USER CODE BEGIN SysInit */

  /* USER CODE END SysInit */

  /* Initialize all configured peripherals */
  MX_GPIO_Init();
  MX_USART2_UART_Init();
  MX_DMA_Init();
  MX_TIM2_Init();

  /* USER CODE BEGIN 2 */

  //Starts for the first time, the DMA input capture mode
  HAL_TIM_IC_Start_DMA(&htim2, TIM_CHANNEL_1, tab_val_DMA, TIM_DMA_Buffer_SIZE);

  /* USER CODE END 2 */

  /* Infinite loop */
  /* USER CODE BEGIN WHILE */

  while (1)
  {
	  /*if I have the TCIF5 and HTCF5 flags set, I process my data:*/
	  if(((hdma_tim2_ch1.DmaBaseAddress->ISR) &= (DMA_ISR_TCIF1 << hdma_tim2_ch1.ChannelIndex)) == (DMA_ISR_TCIF1 << hdma_tim2_ch1.ChannelIndex))//transfer process is done
	  {
		  /*First of all, I initialize all tables, except the DMA buffer table:*/
		  initialise_tab(tab_vals, TIM_DMA_FRAME_ticks_SIZE);
		  initialise_tab(TICKS_tab_DMA, TIM_DMA_Buffer_SIZE);
		  initialise_tab(SENT_data_tab, SENT_data_SIZE);

		  //start of data processing
		  BufferDMA_to_ticks(tab_val_DMA , TICKS_tab_DMA);
		  tick_data_process(tab_val_DMA,tab_vals);
		  data_SENT(tab_vals, SENT_data_tab);
		  //end of data processing

		  //check if the  frame is OK i.e starts with synchro data:
		  flag_frame_ok = set_flag_data_ok(tab_vals);

		  //initialization of the DMA buffer table:

		  initialise_tab(tab_val_DMA, TIM_DMA_Buffer_SIZE);

		  counter_DMA_requests++;//I count the number of DMA requests

		  DMA_available =  HAL_TIM_IC_Stop_DMA(&htim2, TIM_CHANNEL_1);//to change the channel state to ready
		  while(DMA_available == HAL_BUSY){}//for caution, so as to not restart the DMA before it to be  available

		  HAL_TIM_IC_Start_DMA(&htim2, TIM_CHANNEL_1, tab_val_DMA, TIM_DMA_Buffer_SIZE);//I restart the DMA when all the conditions are met
	  }
	  if(flag_frame_ok)
	  {
		  NB_frames_ok++;//I count the number of frame that are ok
		  flag_frame_ok = 0;//reset the flag
	  }

  }
    /* USER CODE END WHILE */

    /* USER CODE BEGIN 3 */

  /* USER CODE END 3 */
}

/**
  * @brief System Clock Configuration
  * @retval None
  */
void SystemClock_Config(void)
{
  RCC_OscInitTypeDef RCC_OscInitStruct = {0};
  RCC_ClkInitTypeDef RCC_ClkInitStruct = {0};

  /** Initializes the RCC Oscillators according to the specified parameters
  * in the RCC_OscInitTypeDef structure.
  */
  RCC_OscInitStruct.OscillatorType = RCC_OSCILLATORTYPE_HSI;
  RCC_OscInitStruct.HSIState = RCC_HSI_ON;
  RCC_OscInitStruct.HSICalibrationValue = RCC_HSICALIBRATION_DEFAULT;
  RCC_OscInitStruct.PLL.PLLState = RCC_PLL_ON;
  RCC_OscInitStruct.PLL.PLLSource = RCC_PLLSOURCE_HSI_DIV2;
  RCC_OscInitStruct.PLL.PLLMUL = RCC_PLL_MUL16;
  if (HAL_RCC_OscConfig(&RCC_OscInitStruct) != HAL_OK)
  {
    Error_Handler();
  }
  /** Initializes the CPU, AHB and APB buses clocks
  */
  RCC_ClkInitStruct.ClockType = RCC_CLOCKTYPE_HCLK|RCC_CLOCKTYPE_SYSCLK
                              |RCC_CLOCKTYPE_PCLK1|RCC_CLOCKTYPE_PCLK2;
  RCC_ClkInitStruct.SYSCLKSource = RCC_SYSCLKSOURCE_PLLCLK;
  RCC_ClkInitStruct.AHBCLKDivider = RCC_SYSCLK_DIV1;
  RCC_ClkInitStruct.APB1CLKDivider = RCC_HCLK_DIV2;
  RCC_ClkInitStruct.APB2CLKDivider = RCC_HCLK_DIV1;

  if (HAL_RCC_ClockConfig(&RCC_ClkInitStruct, FLASH_LATENCY_2) != HAL_OK)
  {
    Error_Handler();
  }
}

/**
  * @brief TIM2 Initialization Function
  * @param None
  * @retval None
  */
static void MX_TIM2_Init(void)
{

  /* USER CODE BEGIN TIM2_Init 0 */

  /* USER CODE END TIM2_Init 0 */

  TIM_MasterConfigTypeDef sMasterConfig = {0};
  TIM_IC_InitTypeDef sConfigIC = {0};

  /* USER CODE BEGIN TIM2_Init 1 */

  /* USER CODE END TIM2_Init 1 */
  htim2.Instance = TIM2;
  htim2.Init.Prescaler = 0;
  htim2.Init.CounterMode = TIM_COUNTERMODE_UP;
  htim2.Init.Period = 65535;
  htim2.Init.ClockDivision = TIM_CLOCKDIVISION_DIV1;
  htim2.Init.AutoReloadPreload = TIM_AUTORELOAD_PRELOAD_ENABLE;
  if (HAL_TIM_IC_Init(&htim2) != HAL_OK)
  {
    Error_Handler();
  }
  sMasterConfig.MasterOutputTrigger = TIM_TRGO_RESET;
  sMasterConfig.MasterSlaveMode = TIM_MASTERSLAVEMODE_DISABLE;
  if (HAL_TIMEx_MasterConfigSynchronization(&htim2, &sMasterConfig) != HAL_OK)
  {
    Error_Handler();
  }
  sConfigIC.ICPolarity = TIM_INPUTCHANNELPOLARITY_FALLING;
  sConfigIC.ICSelection = TIM_ICSELECTION_DIRECTTI;
  sConfigIC.ICPrescaler = TIM_ICPSC_DIV1;
  sConfigIC.ICFilter = 0;
  if (HAL_TIM_IC_ConfigChannel(&htim2, &sConfigIC, TIM_CHANNEL_1) != HAL_OK)
  {
    Error_Handler();
  }
  /* USER CODE BEGIN TIM2_Init 2 */

  /* USER CODE END TIM2_Init 2 */

}

/**
  * @brief USART2 Initialization Function
  * @param None
  * @retval None
  */
static void MX_USART2_UART_Init(void)
{

  /* USER CODE BEGIN USART2_Init 0 */

  /* USER CODE END USART2_Init 0 */

  /* USER CODE BEGIN USART2_Init 1 */

  /* USER CODE END USART2_Init 1 */
  huart2.Instance = USART2;
  huart2.Init.BaudRate = 115200;
  huart2.Init.WordLength = UART_WORDLENGTH_8B;
  huart2.Init.StopBits = UART_STOPBITS_1;
  huart2.Init.Parity = UART_PARITY_NONE;
  huart2.Init.Mode = UART_MODE_TX_RX;
  huart2.Init.HwFlowCtl = UART_HWCONTROL_NONE;
  huart2.Init.OverSampling = UART_OVERSAMPLING_16;
  if (HAL_UART_Init(&huart2) != HAL_OK)
  {
    Error_Handler();
  }
  /* USER CODE BEGIN USART2_Init 2 */

  /* USER CODE END USART2_Init 2 */

}

/**
  * Enable DMA controller clock
  */
static void MX_DMA_Init(void)
{

  /* DMA controller clock enable */
  __HAL_RCC_DMA1_CLK_ENABLE();

}

/**
  * @brief GPIO Initialization Function
  * @param None
  * @retval None
  */
static void MX_GPIO_Init(void)
{
  GPIO_InitTypeDef GPIO_InitStruct = {0};

  /* GPIO Ports Clock Enable */
  __HAL_RCC_GPIOC_CLK_ENABLE();
  __HAL_RCC_GPIOD_CLK_ENABLE();
  __HAL_RCC_GPIOA_CLK_ENABLE();
  __HAL_RCC_GPIOB_CLK_ENABLE();

  /*Configure GPIO pin Output Level */
  HAL_GPIO_WritePin(LD2_GPIO_Port, LD2_Pin, GPIO_PIN_RESET);

  /*Configure GPIO pin Output Level */
  HAL_GPIO_WritePin(GPIOB, GPIO_PIN_10, GPIO_PIN_RESET);

  /*Configure GPIO pin : B1_Pin */
  GPIO_InitStruct.Pin = B1_Pin;
  GPIO_InitStruct.Mode = GPIO_MODE_IT_RISING;
  GPIO_InitStruct.Pull = GPIO_NOPULL;
  HAL_GPIO_Init(B1_GPIO_Port, &GPIO_InitStruct);

  /*Configure GPIO pin : LD2_Pin */
  GPIO_InitStruct.Pin = LD2_Pin;
  GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
  GPIO_InitStruct.Pull = GPIO_NOPULL;
  GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
  HAL_GPIO_Init(LD2_GPIO_Port, &GPIO_InitStruct);

  /*Configure GPIO pin : PB10 */
  GPIO_InitStruct.Pin = GPIO_PIN_10;
  GPIO_InitStruct.Mode = GPIO_MODE_OUTPUT_PP;
  GPIO_InitStruct.Pull = GPIO_NOPULL;
  GPIO_InitStruct.Speed = GPIO_SPEED_FREQ_LOW;
  HAL_GPIO_Init(GPIOB, &GPIO_InitStruct);

  /* EXTI interrupt init*/
  HAL_NVIC_SetPriority(EXTI15_10_IRQn, 0, 0);
  HAL_NVIC_EnableIRQ(EXTI15_10_IRQn);

}

/* USER CODE BEGIN 4 */

void tick_data_process(uint32_t* tab_src,  uint32_t* tab_dst)
{
	uint32_t b = 0;
	uint32_t p = 0;
	uint32_t IC_Val1 = 0;
	uint32_t IC_Val2 = 0;
	uint32_t Difference = 0;
	uint32_t tab_buffer[TIM_DMA_Buffer_SIZE];

	double  usWidth = 0;
	double refClock = 0;
	double mFactor = 0 ;

	refClock = TIMFREQ/(PRESCALAR);
	mFactor = 1000000/refClock;

	for(uint32_t j = 0; j < TIM_DMA_Buffer_SIZE; j++)
	{
		IC_Val1 = tab_src[j];//read the first value
		p = j + 1;
		IC_Val2 = tab_src[p];  // read the second value
		if (IC_Val2 > IC_Val1)
		{
			  Difference = IC_Val2-IC_Val1;

		}
		else if (IC_Val1 > IC_Val2)
		{
			Difference = (65535 - IC_Val1) + IC_Val2;

		}
		usWidth = Difference*mFactor;
		tab_buffer[b] = compute_tick(usWidth,tab_ticks);//this only gets 18 values-->18 differences of 19 falling edges values
		b++;
	}
	fill_table_ticks(tab_buffer, tab_dst, TIM_DMA_Buffer_SIZE);

	initialise_tab(tab_buffer, TIM_DMA_Buffer_SIZE);

}
void initialise_tab(uint32_t* tab, int len_tab)
{
	for(int n = 0; n < len_tab ; n++)
	{
		tab[n] = 0;
	}
}
uint32_t compute_tick(float _width, float TAB_ticks[])
{
	float _quotient =  0;
	float _min = 0;
	float _diff = 0;
	float _val_abs = 0;
	float tick_time = 3.0;
	int indice_min = 0;
	_quotient = _width/TAB_ticks[indice_min];
	_diff = tick_time - _quotient;
	_val_abs = absolute_val(_diff);
	_min = _val_abs;
	for(int i = 1; i < SIZE_TAB_ticks; i++)
	{
		_quotient = _width/TAB_ticks[i];
		_diff = 3.0 - _quotient;
		_val_abs = absolute_val(_diff);
		if(_min > _val_abs )
		{
			_min = _val_abs;
			indice_min = i;
		}

	}
	return (uint32_t)TAB_ticks[indice_min];
}
uint8_t set_flag_data_ok(uint32_t* tab_ticks)
{
	uint8_t flag_ticks_ok = 0;

	if(tab_ticks[0] == TICKS_OF_SYNCHRO)
	{
		flag_ticks_ok = 1;//set the flag if the start of the  frame is 56 ticks
	}
	return flag_ticks_ok;
}
void fill_table_ticks(uint32_t* tab_source, uint32_t* tab_dest, uint32_t size_buffer)
{
	uint32_t k = 0;
	uint32_t n = 0;
	for(n = 0; n < size_buffer; n++)
	{
		k = n+1;
		if(tab_source[n] == TICKS_OF_SYNCHRO)
		{
			if(tab_source[k] == TICKS_OF_SYNCHRO)
			{
				n++;
			}
			for(uint32_t l = 0; l < TIM_DMA_FRAME_ticks_SIZE ; l++)
			{
				tab_dest[l] = tab_source[n];
				if(n < size_buffer)
				{
					n++;
				}
				else
				{
					return;//in case we went through all the elements of the source table
				}

			}
			return;


		}

	}


}
void data_SENT(uint32_t* tab_DMA_ticks, uint32_t* tab_data_SENT)
{
	uint32_t i = 0;//index to be used in table data processing
	if(tab_DMA_ticks[i] == 56)//when we are sure to have a SENT data frame
	{
		i++;//start process data after the synchro
		for(uint32_t j = 0; j < SENT_data_SIZE; j++)
		{
			tab_data_SENT[j] = tab_DMA_ticks[i] - LOWEST_TICKS;
		}
	}
}
void BufferDMA_to_ticks(uint32_t* tab_1 , uint32_t* tab_2)
{
		uint32_t b = 0;
		uint32_t p = 0;
		uint32_t IC_Val1 = 0;
		uint32_t IC_Val2 = 0;
		uint32_t Difference = 0;

		double  usWidth = 0;
		double refClock = 0;
		double mFactor = 0 ;

		refClock = TIMFREQ/(PRESCALAR);
		mFactor = 1000000/refClock;

		for(uint32_t j = 0; j < TIM_DMA_Buffer_SIZE; j++)
		{
			IC_Val1 = tab_1[j];//read the first value
			p = j + 1;
			IC_Val2 = tab_1[p];  // read the second value
			if (IC_Val2 > IC_Val1)
			{
				  Difference = IC_Val2-IC_Val1;

			}
			else if (IC_Val1 > IC_Val2)
			{
				Difference = (65535 - IC_Val1) + IC_Val2;

			}
			usWidth = Difference*mFactor;
			tab_2[b] = compute_tick(usWidth,tab_ticks);//this only gets 18 values-->18 differences of 19 falling edges values
			b++;
		}
}
float absolute_val(float x)
{
	float val_abs;
	if (x < 0)
	{
		val_abs = -x;
	}
	else
		val_abs = x;

	return val_abs;
}


/* USER CODE END 4 */

/**
  * @brief  This function is executed in case of error occurrence.
  * @retval None
  */
void Error_Handler(void)
{
  /* USER CODE BEGIN Error_Handler_Debug */
  /* User can add his own implementation to report the HAL error return state */
  __disable_irq();
  while (1)
  {
  }
  /* USER CODE END Error_Handler_Debug */
}

#ifdef  USE_FULL_ASSERT
/**
  * @brief  Reports the name of the source file and the source line number
  *         where the assert_param error has occurred.
  * @param  file: pointer to the source file name
  * @param  line: assert_param error line source number
  * @retval None
  */
void assert_failed(uint8_t *file, uint32_t line)
{
  /* USER CODE BEGIN 6 */
  /* User can add his own implementation to report the file name and line number,
     ex: printf("Wrong parameters value: file %s on line %d\r\n", file, line) */
  /* USER CODE END 6 */
}
#endif /* USE_FULL_ASSERT */

/************************ (C) COPYRIGHT STMicroelectronics *****END OF FILE****/
